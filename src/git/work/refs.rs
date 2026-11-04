use super::*;
use crate::git::{BranchAction, RefKind, Reference, Refs, refs::Request};

impl Runner {
    pub(super) fn refs(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::Observe(observation) => {
                let (identity, status) = self.observe(admission)?;
                if identity != observation.identity {
                    return Err("Refs repository identity changed".into());
                }
                let id = self.progress.lock().expect("Git progress owner").operation;
                let mut current = Observation {
                    context: observation.context,
                    generation: id.0,
                    identity,
                    status,
                    admission: admission.split(0),
                };
                current.admission = admission.split(current.bytes());
                let refs = self.read_refs(Arc::new(current), admission)?;
                Ok(Candidate::Refs {
                    refs,
                    changed: true,
                })
            }
            Request::Branch { refs, action, .. } => self.branch(refs, action, admission),
        }
    }
    fn check_ref_origin(
        &mut self,
        expected: &Observation,
        admission: &mut Lease,
    ) -> Result<Option<Status>, String> {
        let (identity, status) = self.observe(admission)?;
        if identity != expected.identity
            || status.as_ref().map(|s| &s.head) != expected.status.as_ref().map(|s| &s.head)
        {
            return Err("Refs repository or HEAD changed; refresh Status and Refs".into());
        }
        Ok(status)
    }
    fn read_refs(
        &mut self,
        observation: Arc<Observation>,
        admission: &mut Lease,
    ) -> Result<Refs, String> {
        let before = self.ref_head(observation.identity.format)?;
        let count = format!("--count={}", self.policy.max_refs + 1);
        let bytes=self.command(&["for-each-ref", "--sort=refname", &count,
            "--format=%(refname)%00%(objectname)%00%(objecttype)%00%(symref)%00%(*objectname)%00%(*objecttype)%00%(upstream)%00%(upstream:track)%00",
            "refs/heads/","refs/remotes/","refs/tags/"],Phase::Refreshing,false)?;
        let count = bytes
            .iter()
            .filter(|b| **b == b'\n')
            .count()
            .min(self.policy.max_refs);
        admission.grow(count * std::mem::size_of::<Reference>() + 2 * bytes.len())?;
        let mut entries = Vec::with_capacity(count);
        let mut complete = true;
        for line in bytes.split_inclusive(|b| *b == b'\n') {
            if entries.len() == self.policy.max_refs {
                complete = false;
                break;
            }
            let line = line
                .strip_suffix(b"\0\n")
                .ok_or("Refs record lacks its NUL/newline boundary")?;
            let mut parts = line.split(|b| *b == 0);
            let fields = std::array::from_fn::<_, 8, _>(|_| parts.next().unwrap_or(&[]));
            if parts.next().is_some()
                || line.iter().filter(|b| **b == 0).count() != 7
                || fields.iter().any(|f| f.len() > 4096)
            {
                return Err("Ref metadata exceeds its exact field bound".into());
            }
            let kind = if fields[0].starts_with(b"refs/heads/") {
                RefKind::Branch
            } else if fields[0].starts_with(b"refs/remotes/") {
                RefKind::Tracking
            } else if fields[0].starts_with(b"refs/tags/") {
                RefKind::Tag
            } else {
                return Err("Unexpected ref namespace".into());
            };
            entries.push(Reference {
                name: fields[0].to_vec(),
                kind,
                object: ObjectId::parse(fields[1], observation.identity.format)?,
                object_type: fields[2].to_vec(),
                symbolic: fields[3].to_vec(),
                peeled: if fields[4].is_empty() {
                    None
                } else {
                    Some(ObjectId::parse(fields[4], observation.identity.format)?)
                },
                peeled_type: fields[5].to_vec(),
                upstream: fields[6].to_vec(),
                tracking: fields[7].to_vec(),
            });
        }
        // Config keys disclose configured names without retaining URL values,
        // including credential-bearing remote descriptions.
        let keys = self.command(
            &[
                "config",
                "--null",
                "--name-only",
                "--get-regexp",
                "^remote\\..*\\.",
            ],
            Phase::Refreshing,
            true,
        )?;
        admission
            .grow(2 * keys.len() + self.policy.max_remotes * std::mem::size_of::<Vec<u8>>())?;
        let mut remotes = Vec::with_capacity(self.policy.max_remotes);
        if !keys.is_empty() && keys.last() != Some(&0) {
            return Err("Remote key list is incomplete".into());
        }
        for key in keys.split(|b| *b == 0).filter(|k| !k.is_empty()) {
            let key = key
                .strip_prefix(b"remote.")
                .ok_or("Malformed configured remote key")?;
            let end = key
                .iter()
                .rposition(|b| *b == b'.')
                .ok_or("Missing remote name boundary")?;
            let name = &key[..end];
            if name.is_empty() || name.len() > 4096 {
                return Err("Configured remote name exceeds metadata bound".into());
            }
            if remotes.iter().any(|r| r == name) {
                continue;
            }
            if remotes.len() == self.policy.max_remotes {
                complete = false;
                break;
            }
            remotes.push(name.to_vec());
        }
        let after = self.ref_head(observation.identity.format)?;
        if before != after {
            return Err("HEAD changed during ref observation".into());
        }
        let mut refs = Refs {
            id: self.progress.lock().expect("Git progress owner").operation,
            observation,
            entries,
            remotes,
            head_ref: before.0,
            head: before.1,
            completeness: if complete {
                Completeness::Complete
            } else {
                Completeness::Truncated {
                    reason: "max_refs or max_remotes",
                }
            },
            _admission: admission.split(0),
        };
        refs._admission = admission.split(refs.bytes());
        Ok(refs)
    }
    fn ref_head(&mut self, format: ObjectFormat) -> Result<(Vec<u8>, Option<ObjectId>), String> {
        let reference =
            self.command(&["symbolic-ref", "--quiet", "HEAD"], Phase::Verifying, true)?;
        let reference = reference.strip_suffix(b"\n").unwrap_or(&reference);
        if reference.len() > 4096 {
            return Err("HEAD ref exceeds metadata bound".into());
        }
        let object = self.command(
            &["rev-parse", "--verify", "--quiet", "HEAD"],
            Phase::Verifying,
            true,
        )?;
        let object = object.strip_suffix(b"\n").unwrap_or(&object);
        Ok((
            reference.to_vec(),
            if object.is_empty() {
                None
            } else {
                Some(ObjectId::parse(object, format)?)
            },
        ))
    }
    fn branch(
        &mut self,
        refs: Arc<Refs>,
        action: BranchAction,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let status = self.check_ref_origin(&refs.observation, admission)?;
        let before = self.read_refs(refs.observation.clone(), admission)?;
        if !refs.same(&before) {
            return Err("Refs changed while confirmation was open; refresh required".into());
        }
        let selected = match &action {
            BranchAction::Create { .. } => None,
            BranchAction::Switch { name }
            | BranchAction::Rename { name, .. }
            | BranchAction::Delete { name } => refs.reference(name),
        };
        let short = selected.map(|r| OsString::from_vec(r.name[b"refs/heads/".len()..].to_vec()));
        let mut args: Vec<OsString> = Vec::new();
        match &action {
            BranchAction::Create { name, target } => {
                self.branch_name(name)?;
                if refs
                    .reference(format!("refs/heads/{name}").as_bytes())
                    .is_some()
                {
                    return Err("Branch name already exists".into());
                }
                let actual = self.command(
                    &[
                        "rev-parse",
                        "--verify",
                        "--end-of-options",
                        &format!("{}^{{commit}}", target.hex()),
                    ],
                    Phase::Checking,
                    false,
                )?;
                if actual.strip_suffix(b"\n") != Some(target.hex().as_bytes()) {
                    return Err("Branch target is not the exact admitted commit".into());
                }
                args.extend([
                    "branch".into(),
                    "--no-track".into(),
                    "--".into(),
                    name.into(),
                    target.hex().into(),
                ]);
            }
            BranchAction::Switch { .. } => {
                let status = status.as_ref().ok_or("Switch requires a working tree")?;
                if status.completeness != Completeness::Complete || !status.paths.is_empty() {
                    return Err(
                        "Branch switch requires a complete clean index and working tree".into(),
                    );
                }
                args.extend([
                    "switch".into(),
                    "--no-guess".into(),
                    "--".into(),
                    short.clone().ok_or("Missing branch")?,
                ]);
            }
            BranchAction::Rename { new_name, .. } => {
                self.branch_name(new_name)?;
                if refs
                    .reference(format!("refs/heads/{new_name}").as_bytes())
                    .is_some()
                {
                    return Err("Branch name already exists".into());
                }
                args.extend([
                    "branch".into(),
                    "--move".into(),
                    "--".into(),
                    short.clone().ok_or("Missing branch")?,
                    new_name.into(),
                ]);
            }
            BranchAction::Delete { name } => {
                if refs.head_ref == *name {
                    return Err("Cannot delete the current branch".into());
                }
                let object = selected.ok_or("Missing branch")?.object.hex();
                let head = refs
                    .head
                    .ok_or("Branch deletion requires a checked HEAD")?
                    .hex();
                self.command(
                    &["merge-base", "--is-ancestor", &object, &head],
                    Phase::Checking,
                    false,
                )?;
                args.extend([
                    "branch".into(),
                    "--delete".into(),
                    "--".into(),
                    short.clone().ok_or("Missing branch")?,
                ]);
            }
        }
        let status = self.check_ref_origin(&refs.observation, admission)?;
        if matches!(action, BranchAction::Switch { .. })
            && status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete || !s.paths.is_empty())
        {
            return Err("Branch switch disk state changed before launch".into());
        }
        let checked = self.read_refs(refs.observation.clone(), admission)?;
        if !refs.same(&checked) {
            return Err("Refs changed before branch execution".into());
        }
        let (executed, facts) = self.execute_change(&args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Branch cleanup remains owned; effects need inspection".into());
        }
        let result = (|| {
            let (identity, status) = self.observe(admission)?;
            let progress = *self.progress.lock().expect("Git progress owner");
            let mut observation = Observation {
                context: refs.observation.context,
                generation: progress.operation.0,
                identity,
                status,
                admission: admission.split(0),
            };
            observation.admission = admission.split(observation.bytes());
            let observation = Arc::new(observation);
            let after = self.read_refs(observation, admission)?;
            let valid = after.observation.identity == refs.observation.identity
                && match &action {
                    BranchAction::Create { name, target } => {
                        after
                            .reference(format!("refs/heads/{name}").as_bytes())
                            .is_some_and(|r| r.object == *target)
                            && after.head_ref == refs.head_ref
                            && after.head == refs.head
                    }
                    BranchAction::Switch { name } => {
                        after.head_ref == *name && after.head == selected.map(|r| r.object)
                    }
                    BranchAction::Rename { name, new_name } => {
                        after.reference(name).is_none()
                            && after
                                .reference(format!("refs/heads/{new_name}").as_bytes())
                                .is_some_and(|r| Some(r.object) == selected.map(|r| r.object))
                            && after.head == refs.head
                            && after.head_ref
                                == if refs.head_ref == *name {
                                    format!("refs/heads/{new_name}").into_bytes()
                                } else {
                                    refs.head_ref.clone()
                                }
                    }
                    BranchAction::Delete { name } => {
                        after.reference(name).is_none()
                            && after.head == refs.head
                            && after.head_ref == refs.head_ref
                    }
                };
            if executed.is_ok() && valid {
                self.effects = Effects::Verified;
            } else {
                self.failure_verified = executed.is_err() && after.same(&before);
                self.operation_failure=Some("Branch child failed or exact postcondition differs; inspect actual diagnostics and refs".into());
            }
            Ok(Candidate::Refs {
                refs: after,
                changed: true,
            })
        })();
        self.restore_execution(facts, executed.is_err());
        if result.is_err() {
            self.operation_failure =
                Some("Branch effects remain unverified; refresh required".into());
        }
        result
    }
    fn branch_name(&mut self, name: &str) -> Result<(), String> {
        let bytes = self.command(
            &["check-ref-format", "--branch", name],
            Phase::Checking,
            false,
        )?;
        if bytes.strip_suffix(b"\n") != Some(name.as_bytes()) {
            return Err("Branch name resolves to a different identity".into());
        }
        Ok(())
    }
}
