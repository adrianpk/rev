use super::*;
use crate::git::{TagIntent, TagKind};

impl Runner {
    fn tag_policy(&mut self) -> Result<(Vec<u8>, bool, bool), String> {
        let bytes = self.command(
            &[
                "config",
                "--null",
                "--get-regexp",
                "^(tag\\.(gpgsign|forcesignannotated)|user\\.signingkey|gpg\\..*)$",
            ],
            Phase::Checking,
            true,
        )?;
        if bytes.split(|b| *b == 0).count() > 65
            || bytes
                .split(|b| *b == 0 || *b == b'\n')
                .any(|v| v.len() > 4096)
        {
            return Err("Tag signing policy exceeds metadata bounds".into());
        }
        let mut flags = [false; 2];
        for (i, key) in ["tag.gpgsign", "tag.forcesignannotated"].iter().enumerate() {
            flags[i] = match self
                .command(&["config", "--bool", "--get", key], Phase::Checking, true)?
                .as_slice()
            {
                b"true\n" => true,
                b"false\n" | b"" => false,
                _ => return Err("Tag signing policy is unavailable".into()),
            };
        }
        Ok((bytes, flags[0], flags[1]))
    }
    pub(super) fn prepare_tag(
        &mut self,
        refs: Arc<Refs>,
        name: String,
        kind: TagKind,
        target: ObjectId,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_ref_snapshot(&refs, admission)?;
        if refs.entries.len() >= self.policy.max_refs {
            return Err("New tag exceeds configured complete-ref admission".into());
        }
        let full = format!("refs/tags/{name}");
        self.command(&["check-ref-format", &full], Phase::Checking, false)?;
        if refs.reference(full.as_bytes()).is_some() {
            return Err("Tag name already exists; replacement is not admitted".into());
        }
        let target_type = self.command(
            &["--no-replace-objects", "cat-file", "-t", &target.hex()],
            Phase::Checking,
            false,
        )?;
        let target_type = target_type
            .strip_suffix(b"\n")
            .ok_or("Tag target type is incomplete")?;
        if ![b"commit".as_slice(), b"tag", b"tree", b"blob"].contains(&target_type) {
            return Err("Unsupported exact tag target type".into());
        }
        admission.grow(16384)?;
        let (policy, sign_all, sign_annotated) = self.tag_policy()?;
        if kind == TagKind::Lightweight && sign_all {
            return Err(
                "Lightweight tag conflicts with tag.gpgSign; explicitly choose annotated or cancel"
                    .into(),
            );
        }
        self.check_ref_snapshot(&refs, admission)?;
        let mut intent = TagIntent {
            refs,
            name,
            kind,
            target,
            target_type: target_type.into(),
            signing: sign_all || (kind == TagKind::Annotated && sign_annotated),
            policy,
            _admission: admission.split(0),
        };
        admission.grow(intent.bytes())?;
        intent._admission = admission.split(intent.bytes());
        Ok(Candidate::TagIntent(intent))
    }
    pub(super) fn create_tag(
        &mut self,
        intent: &Arc<TagIntent>,
        text: &str,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_ref_snapshot(&intent.refs, admission)?;
        let (policy, sign_all, _) = self.tag_policy()?;
        if policy != intent.policy || (intent.kind == TagKind::Lightweight && sign_all) {
            return Err(
                "Tag signing policy changed; explicit preparation and review required".into(),
            );
        }
        let mut args: Vec<OsString> = vec!["--no-replace-objects".into(), "tag".into()];
        let input = if intent.kind == TagKind::Annotated {
            args.extend([
                if intent.signing {
                    "--sign".into()
                } else {
                    "--annotate".into()
                },
                "--cleanup=verbatim".into(),
                "--file=-".into(),
            ]);
            crate::process::input::stage(Some(text.as_bytes()))?
        } else {
            None
        };
        args.extend([
            "--".into(),
            intent.name.clone().into(),
            intent.target.hex().into(),
        ]);
        self.check_ref_snapshot(&intent.refs, admission)?;
        if self.tag_policy()?.0 != intent.policy {
            return Err("Tag signing policy changed before launch; review again".into());
        }
        let (executed, facts) = self.execute_change(&args, input);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Tag child cleanup remains owned; effects need inspection".into());
        }
        let result = (|| {
            let after = self.refresh_ref_result(&intent.refs, admission)?;
            let full = format!("refs/tags/{}", intent.name);
            let valid = match after.reference(full.as_bytes()) {
                Some(r) if intent.kind == TagKind::Lightweight => r.object == intent.target,
                Some(r) if r.object_type == b"tag" => {
                    self.verify_annotation(r, intent, text, admission)?
                }
                _ => false,
            } && after.head == intent.refs.head
                && after.head_ref == intent.refs.head_ref;
            if executed.is_ok() && valid {
                self.effects = Effects::Verified;
            } else {
                self.failure_verified = executed.is_err() && after.same(&intent.refs);
                self.operation_failure=Some("Tag child failed or exact postcondition differs; annotation retained; inspect diagnostics".into());
            }
            Ok(Candidate::Refs {
                refs: after,
                changed: true,
            })
        })();
        self.restore_execution(facts, executed.is_err());
        if result.is_err() {
            self.operation_failure =
                Some("Tag effects remain unverified; annotation retained; refresh required".into());
        }
        result
    }
    fn verify_annotation(
        &mut self,
        reference: &Reference,
        intent: &TagIntent,
        text: &str,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let (bytes, limited) = self.command_os(
            &[
                "--no-replace-objects".into(),
                "cat-file".into(),
                "tag".into(),
                reference.object.hex().into(),
            ],
            Phase::Verifying,
            false,
            Some(self.policy.max_detail_bytes),
        )?;
        if limited {
            return Err("Created tag object exceeds complete verification admission".into());
        }
        admission.grow(2 * bytes.len() + 2 * text.len() + 4096)?;
        let Some(boundary) = bytes.windows(2).position(|b| b == b"\n\n") else {
            return Ok(false);
        };
        let mut fields = bytes[..boundary].split(|b| *b == b'\n');
        let header = fields.next() == Some(format!("object {}", intent.target.hex()).as_bytes())
            && fields.next()
                == Some(
                    [b"type ".as_slice(), &intent.target_type]
                        .concat()
                        .as_slice(),
                )
            && fields.next() == Some(format!("tag {}", intent.name).as_bytes())
            && fields
                .next()
                .is_some_and(|s| s.starts_with(b"tagger ") && s.len() > b"tagger ".len())
            && fields.next().is_none();
        if !header {
            return Ok(false);
        }
        let body = &bytes[boundary + 2..];
        let expected = text.as_bytes();
        let signature = self.command_os(
            &[
                "for-each-ref".into(),
                "--format=%(contents:signature)%00".into(),
                OsString::from_vec(reference.name.clone()),
            ],
            Phase::Verifying,
            false,
            Some(self.policy.max_detail_bytes),
        )?;
        if signature.1 {
            return Err("Tag signature exceeds verification admission".into());
        }
        let signature = signature
            .0
            .strip_suffix(b"\0\n")
            .ok_or("Tag signature response is incomplete")?;
        if intent.signing && signature.is_empty() {
            return Ok(false);
        }
        Ok(body == expected
            || (!signature.is_empty() && body.strip_suffix(signature) == Some(expected)))
    }
    pub(super) fn delete_tag(
        &mut self,
        refs: &Arc<Refs>,
        name: &[u8],
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_ref_snapshot(refs, admission)?;
        let object = refs.reference(name).ok_or("Tag identity expired")?.object;
        let args = [
            OsString::from("update-ref"),
            "--no-deref".into(),
            "-d".into(),
            OsString::from_vec(name.into()),
            object.hex().into(),
        ];
        let (executed, facts) = self.execute_change(&args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Tag deletion cleanup remains owned; inspect actual refs".into());
        }
        let result = (|| {
            let after = self.refresh_ref_result(refs, admission)?;
            if executed.is_ok()
                && after.reference(name).is_none()
                && after.head == refs.head
                && after.head_ref == refs.head_ref
            {
                self.effects = Effects::Verified;
            } else {
                self.failure_verified = executed.is_err() && after.same(refs);
                self.operation_failure = Some(
                    "Exact tag deletion failed or postcondition differs; inspect diagnostics"
                        .into(),
                );
            }
            Ok(Candidate::Refs {
                refs: after,
                changed: true,
            })
        })();
        self.restore_execution(facts, executed.is_err());
        if result.is_err() {
            self.operation_failure = Some("Tag deletion effects remain unverified".into());
        }
        result
    }
}
