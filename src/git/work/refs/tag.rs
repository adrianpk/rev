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
    pub(super) fn create_tag(
        &mut self,
        intent: &Arc<TagIntent>,
        text: &str,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_tag_policy(intent, admission)?;
        let (args, input) = tag_arguments(intent, text)?;
        self.check_ref_snapshot(&intent.refs, admission)?;
        if self.tag_policy()?.0 != intent.policy {
            return Err("Tag signing policy changed before launch; review again".into());
        }
        let (executed, facts) = self.execute_change(&args, input);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Tag child cleanup remains owned; effects need inspection".into());
        }
        let result =
            self.observe_created_tag(&Annotation { intent, text }, executed.is_ok(), admission);
        self.restore_execution(facts, executed.is_err());
        if result.is_err() {
            self.operation_failure =
                Some("Tag effects remain unverified; annotation retained; refresh required".into());
        }
        result
    }
    fn check_tag_policy(
        &mut self,
        intent: &TagIntent,
        admission: &mut Lease,
    ) -> Result<(), String> {
        self.check_ref_snapshot(&intent.refs, admission)?;
        let (policy, sign_all, _) = self.tag_policy()?;
        if policy != intent.policy || (intent.kind == TagKind::Lightweight && sign_all) {
            return Err(
                "Tag signing policy changed; explicit preparation and review required".into(),
            );
        }
        Ok(())
    }
    fn observe_created_tag(
        &mut self,
        annotation: &Annotation<'_>,
        executed: bool,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let after = self.refresh_ref_result(&annotation.intent.refs, admission)?;
        let full = format!("refs/tags/{}", annotation.intent.name);
        let valid = annotation.verify(self, after.reference(full.as_bytes()), admission)?
            && after.head == annotation.intent.refs.head
            && after.head_ref == annotation.intent.refs.head_ref;
        if executed && valid {
            self.effects = Effects::Verified;
        } else {
            self.failure_verified = !executed && after.same(&annotation.intent.refs);
            self.operation_failure=Some("Tag child failed or exact postcondition differs; annotation retained; inspect diagnostics".into());
        }
        Ok(Candidate::Refs {
            refs: after,
            changed: true,
        })
    }
    fn verify_annotation(
        &mut self,
        reference: &Reference,
        intent: &TagIntent,
        text: &str,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let bytes = self.read_tag_object(reference)?;
        admission.grow(2 * bytes.len() + 2 * text.len() + 4096)?;
        let Some(boundary) = bytes.windows(2).position(|b| b == b"\n\n") else {
            return Ok(false);
        };
        if !annotation_header(&bytes[..boundary], intent) {
            return Ok(false);
        }
        let body = &bytes[boundary + 2..];
        let expected = text.as_bytes();
        let signature = self.tag_signature(reference)?;
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
    fn tag_signature(&mut self, reference: &Reference) -> Result<(Vec<u8>, bool), String> {
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
        Ok(signature)
    }
    fn read_tag_object(&mut self, reference: &Reference) -> Result<Vec<u8>, String> {
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
        Ok(bytes)
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
        let result = self.observe_deleted_tag(refs, name, executed.is_ok(), admission);
        self.restore_execution(facts, executed.is_err());
        if result.is_err() {
            self.operation_failure = Some("Tag deletion effects remain unverified".into());
        }
        result
    }
    fn observe_deleted_tag(
        &mut self,
        refs: &Refs,
        name: &[u8],
        executed: bool,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let after = self.refresh_ref_result(refs, admission)?;
        if executed
            && after.reference(name).is_none()
            && after.head == refs.head
            && after.head_ref == refs.head_ref
        {
            self.effects = Effects::Verified;
        } else {
            self.failure_verified = !executed && after.same(refs);
            self.operation_failure = Some(
                "Exact tag deletion failed or postcondition differs; inspect diagnostics".into(),
            );
        }
        Ok(Candidate::Refs {
            refs: after,
            changed: true,
        })
    }
}

/// Exact requested tag identity; preparation adds admitted target and signing proof.
pub(super) struct Draft {
    pub name: String,
    pub kind: TagKind,
    pub target: ObjectId,
}
impl Draft {
    pub(super) fn prepare(
        self,
        runner: &mut Runner,
        refs: Arc<Refs>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        runner.check_ref_snapshot(&refs, admission)?;
        let target_type = self.target_type(runner, &refs)?;
        admission.grow(16384)?;
        let (policy, sign_all, sign_annotated) = runner.tag_policy()?;
        self.check_lightweight(sign_all)?;
        runner.check_ref_snapshot(&refs, admission)?;
        let mut intent = TagIntent {
            refs,
            name: self.name,
            kind: self.kind,
            target: self.target,
            target_type,
            signing: sign_all || (self.kind == TagKind::Annotated && sign_annotated),
            policy,
            _admission: admission.split(0),
        };
        admission.grow(intent.bytes())?;
        intent._admission = admission.split(intent.bytes());
        Ok(Candidate::TagIntent(intent))
    }
    fn check_lightweight(&self, sign_all: bool) -> Result<(), String> {
        if self.kind == TagKind::Lightweight && sign_all {
            return Err(
                "Lightweight tag conflicts with tag.gpgSign; explicitly choose annotated or cancel"
                    .into(),
            );
        }
        Ok(())
    }
    fn target_type(&self, runner: &mut Runner, refs: &Refs) -> Result<Vec<u8>, String> {
        if refs.entries.len() >= runner.policy.max_refs {
            return Err("New tag exceeds configured complete-ref admission".into());
        }
        let full = format!("refs/tags/{}", self.name);
        runner.command(&["check-ref-format", &full], Phase::Checking, false)?;
        if refs.reference(full.as_bytes()).is_some() {
            return Err("Tag name already exists; replacement is not admitted".into());
        }
        let target_type = runner.command(
            &["--no-replace-objects", "cat-file", "-t", &self.target.hex()],
            Phase::Checking,
            false,
        )?;
        let target_type = target_type
            .strip_suffix(b"\n")
            .ok_or("Tag target type is incomplete")?;
        if ![b"commit".as_slice(), b"tag", b"tree", b"blob"].contains(&target_type) {
            return Err("Unsupported exact tag target type".into());
        }
        Ok(target_type.into())
    }
}
struct Annotation<'a> {
    intent: &'a TagIntent,
    text: &'a str,
}
impl Annotation<'_> {
    fn verify(
        &self,
        runner: &mut Runner,
        reference: Option<&Reference>,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        Ok(match reference {
            Some(r) if self.intent.kind == TagKind::Lightweight => r.object == self.intent.target,
            Some(r) if r.object_type == b"tag" => {
                runner.verify_annotation(r, self.intent, self.text, admission)?
            }
            _ => false,
        })
    }
}
fn annotation_header(bytes: &[u8], intent: &TagIntent) -> bool {
    let mut fields = bytes.split(|b| *b == b'\n');
    fields.next() == Some(format!("object {}", intent.target.hex()).as_bytes())
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
        && fields.next().is_none()
}

fn tag_arguments(
    intent: &TagIntent,
    text: &str,
) -> Result<(Vec<OsString>, Option<std::fs::File>), String> {
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
    Ok((args, input))
}
