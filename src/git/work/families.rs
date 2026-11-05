use super::*;
use crate::git::conflicts::resolution::fingerprint;
use crate::git::{Family, FamilyPlan, FamilyResult, families::Request};
mod notes;
mod output;
mod sparse;
mod state;
mod submodule;
mod subtree;

impl Runner {
    pub(super) fn family(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::Prepare {
                family,
                args,
                observation,
                output,
                ..
            } => self.prepare_family(family, args, observation, output, admission),
            Request::Execute(plan) => self.execute_family(plan, admission),
        }
    }
    fn family_destination_stamp(path: &Path) -> Result<[u64; 8], String> {
        if !std::fs::symlink_metadata(path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        {
            return Err("Prepared destination already exists or cannot be checked".into());
        }
        let parent = path.parent().ok_or("Prepared destination has no parent")?;
        let metadata = std::fs::symlink_metadata(parent)
            .map_err(|e| format!("Observe destination parent: {e}"))?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || std::fs::canonicalize(parent).map_err(|e| e.to_string())? != parent
        {
            return Err(
                "Prepared destination parent is not an ordinary canonical directory".into(),
            );
        }
        Ok(fingerprint(&metadata))
    }
    fn family_destination(&self, arg: &std::ffi::OsStr) -> Result<PathBuf, String> {
        let input = Path::new(arg);
        let input = if input.is_absolute() {
            input.to_owned()
        } else {
            self.directory.join(input)
        };
        let mut path = PathBuf::new();
        for part in input.components() {
            match part {
                std::path::Component::RootDir => path.push("/"),
                std::path::Component::Normal(p) => path.push(p),
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir if path.pop() => {}
                _ => return Err("Prepared destination escapes its absolute root".into()),
            }
        }
        if path.file_name().is_none() || path.as_os_str().len() > 4096 {
            return Err("Prepared destination must be a non-root path within 4096 bytes".into());
        }
        Ok(path)
    }
    fn family_relative_destination(&self, arg: &std::ffi::OsStr) -> Result<PathBuf, String> {
        let relative = Path::new(arg);
        if relative
            .components()
            .any(|p| !matches!(p, std::path::Component::Normal(n) if n != ".git"))
        {
            return Err(
                "Prepared repository path requires literal relative components outside .git".into(),
            );
        }
        let path = self.directory.join(relative);
        if path.as_os_str().len() > 4096 {
            return Err("Prepared repository path exceeds 4096 bytes".into());
        }
        // Check every existing component. Git must not follow a replaced
        // symlink into an unrelated directory, including for an empty child.
        let mut current = self.directory.clone();
        for component in relative.components() {
            current.push(component);
            match std::fs::symlink_metadata(&current) {
                Ok(m) if m.is_dir() && !m.file_type().is_symlink() => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
                _ => {
                    return Err("Prepared repository path is not an ordinary directory path".into());
                }
            }
        }
        Ok(path)
    }

    fn prepare_family(
        &mut self,
        family: Family,
        mut args: Vec<OsString>,
        observation: Option<Arc<Observation>>,
        output: Option<PathBuf>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        family.validate(&args)?;
        self.directory = std::fs::canonicalize(&self.directory)
            .map_err(|e| format!("Resolve prepared command directory: {e}"))?;
        let directory_stamp =
            fingerprint(&std::fs::metadata(&self.directory).map_err(|e| e.to_string())?);
        if !self.directory.is_dir() {
            return Err("Prepared command requires a captured directory".into());
        }
        let commands = self.command(&["--list-cmds=main,others"], Phase::Checking, false)?;
        if !commands
            .split(|b| *b == b'\n')
            .any(|name| name == args[0].as_encoded_bytes())
        {
            return Err("Installed Git does not provide the selected command family".into());
        }
        let destination = if let Some(output) = output {
            self.family_destination(output.as_os_str())?
        } else if family.notes() || family.sparse() {
            self.directory.clone()
        } else if family.subtree() {
            self.family_relative_destination(&args[3])?
        } else if family.submodule() {
            if family == Family::SubmoduleList {
                self.directory.clone()
            } else {
                self.family_relative_destination(args.last().ok_or("Submodule path is required")?)?
            }
        } else {
            self.family_destination(args.last().ok_or("Destination is required")?)?
        };
        let parent_stamp = if !family.repository_action()
            || family.output()
            || matches!(family, Family::SubmoduleAdd | Family::SubtreeAdd)
        {
            Some(Self::family_destination_stamp(&destination)?)
        } else {
            None
        };
        if !family.repository_action() {
            *args.last_mut().expect("checked destination") = destination.as_os_str().into();
        }
        let source = observation
            .as_ref()
            .map(|o| {
                let refs = self.integration_refs(o, admission)?;
                let expected = refs
                    .observation
                    .identity
                    .worktree
                    .as_ref()
                    .unwrap_or(&refs.observation.identity.git_directory);
                if expected != &self.directory {
                    return Err(
                        "Prepared directory differs from the captured repository root".into(),
                    );
                }
                self.read_sequence(refs, admission).map(Arc::new)
            })
            .transpose()?;
        let evidence = if let Some(source) = source
            .as_ref()
            .filter(|s| s.refs.observation.status.is_some())
        {
            self.stash_evidence(source, admission)?
        } else {
            [Vec::new(), Vec::new(), Vec::new()]
        };
        let submodule = if family.submodule() {
            let source = source
                .as_ref()
                .ok_or("Submodule source context is unavailable")?;
            if source.family.is_some()
                || source
                    .refs
                    .observation
                    .status
                    .as_ref()
                    .is_none_or(|s| s.completeness != Completeness::Complete)
            {
                return Err(
                    "Submodule preparation requires complete status and no active sequence".into(),
                );
            }
            let state = self.submodule_state(family, &destination, source, admission)?;
            args.insert(0, "--literal-pathspecs".into());
            Some(state)
        } else {
            None
        };
        let subtree = if family.subtree() {
            Some(
                self.prepare_subtree(
                    family,
                    &mut args,
                    &destination,
                    source
                        .as_ref()
                        .ok_or("Subtree source context is unavailable")?,
                    admission,
                )?,
            )
        } else {
            None
        };
        let proof = if let Some(state) = submodule {
            crate::git::families::FamilyProof::Submodule(state)
        } else if let Some(state) = subtree {
            crate::git::families::FamilyProof::Subtree(state)
        } else if family.output() {
            crate::git::families::FamilyProof::Output(self.prepare_output(
                family,
                &mut args,
                &destination,
                source.as_ref().ok_or("Output source context unavailable")?,
                admission,
            )?)
        } else if family.notes() {
            crate::git::families::FamilyProof::Notes(self.prepare_notes(
                family,
                &mut args,
                source.as_ref().ok_or("Notes source context unavailable")?,
                admission,
            )?)
        } else if family.sparse() {
            crate::git::families::FamilyProof::Sparse(self.prepare_sparse(
                family,
                &args,
                source.as_ref().ok_or("Sparse source context unavailable")?,
                admission,
            )?)
        } else {
            crate::git::families::FamilyProof::Creation
        };
        let size = std::mem::size_of::<FamilyPlan>()
            + args.capacity() * std::mem::size_of::<OsString>()
            + args.iter().map(OsString::capacity).sum::<usize>()
            + evidence.iter().map(Vec::capacity).sum::<usize>()
            + self.directory.capacity()
            + destination.capacity()
            + self.policy.program.capacity()
            + proof.bytes()
            + 4096;
        admission.grow(size)?;
        Ok(Candidate::FamilyPlan(FamilyPlan {
            family,
            directory: self.directory.clone(),
            program: self.policy.program.clone(),
            args,
            destination,
            observation,
            source,
            evidence,
            directory_stamp,
            parent_stamp,
            proof,
            _admission: admission.split(size),
        }))
    }
    fn execute_family(
        &mut self,
        plan: Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        if std::fs::canonicalize(&plan.directory).map_err(|e| e.to_string())? != plan.directory
            || fingerprint(&std::fs::metadata(&plan.directory).map_err(|e| e.to_string())?)
                != plan.directory_stamp
            || plan.parent_stamp.is_some_and(|expected| {
                !Self::family_destination_stamp(&plan.destination)
                    .is_ok_and(|actual| actual == expected)
            })
        {
            return Err(
                "Prepared directory or destination parent changed while review was open".into(),
            );
        }
        if let Some(source) = &plan.source {
            self.checked_sequence(source, admission)?;
            if source.refs.observation.status.is_some()
                && self.stash_evidence(source, admission)? != plan.evidence
            {
                return Err("Saved source bytes changed while prepared review was open".into());
            }
        }
        if let Some(expected) = plan.submodule() {
            let actual = self.submodule_state(
                plan.family,
                &plan.destination,
                plan.source.as_ref().expect("submodule source"),
                admission,
            )?;
            if !expected.same(&actual) {
                return Err("Submodule configuration, gitlink or saved child state changed while review was open".into());
            }
            return self.execute_submodule(plan, admission);
        }
        if plan.subtree().is_some() {
            return self.execute_subtree(plan, admission);
        }
        if matches!(plan.proof, crate::git::families::FamilyProof::Output(_)) {
            return self.execute_output(plan, admission);
        }
        if matches!(plan.proof, crate::git::families::FamilyProof::Notes(_)) {
            return self.execute_notes(plan, admission);
        }
        if matches!(plan.proof, crate::git::families::FamilyProof::Sparse(_)) {
            return self.execute_sparse(plan, admission);
        }
        let (executed, facts) = self.execute_change(&plan.args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Prepared Git cleanup remains owned; no destination deletion or retry was attempted".into());
        }
        let result = (|| {
            executed?;
            self.directory = plan.destination.clone();
            let (created, status) = self.observe(admission)?;
            let bare = matches!(plan.family, Family::CloneBare | Family::CloneMirror);
            if created.worktree.is_none() != bare
                || (!bare && created.worktree.as_ref() != Some(&plan.destination))
            {
                return Err(
                    "Created repository identity does not match the reviewed destination and kind"
                        .into(),
                );
            }
            if plan.family == Family::Init
                && !status
                    .as_ref()
                    .is_some_and(|s| matches!(s.head, Head::Unborn { .. }))
            {
                return Err("Initialized destination is not a verified unborn repository".into());
            }
            if plan.family != Family::Init {
                let head = self.command(
                    &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"],
                    Phase::Verifying,
                    true,
                )?;
                // Empty remotes legitimately have no HEAD commit. Existing
                // objects still require a complete format-checked identity.
                if !head.is_empty() {
                    ObjectId::parse(
                        head.strip_suffix(b"\n")
                            .ok_or("Clone HEAD has no newline")?,
                        created.format,
                    )?;
                }
                if plan.family == Family::CloneShallow
                    && self.command(
                        &["rev-parse", "--is-shallow-repository"],
                        Phase::Verifying,
                        false,
                    )? != b"true\n"
                {
                    return Err("Clone did not create the requested shallow repository".into());
                }
                if plan.family == Family::CloneMirror
                    && self.command(
                        &["config", "--bool", "remote.origin.mirror"],
                        Phase::Verifying,
                        false,
                    )? != b"true\n"
                {
                    return Err("Clone did not retain mirror configuration".into());
                }
            }
            self.directory = plan.directory.clone();
            let source = if let Some(expected) = &plan.observation {
                let (identity, status) = self.observe(admission)?;
                if identity != expected.identity {
                    return Err("Source repository identity changed after the command".into());
                }
                let mut source = Observation {
                    context: expected.context,
                    generation: self.progress.lock().expect("Git progress").operation.0,
                    identity,
                    status,
                    admission: admission.split(0),
                };
                let size = source.bytes();
                admission.grow(size)?;
                source.admission = admission.split(size);
                Some(source)
            } else {
                None
            };
            let size = std::mem::size_of::<FamilyResult>() + created.bytes() + 4096;
            admission.grow(size)?;
            self.effects = Effects::Verified;
            Ok(Candidate::FamilyResult(FamilyResult {
                plan,
                created: Some(created),
                object: None,
                source,
                _admission: admission.split(size),
            }))
        })();
        if let Err(e) = &result {
            self.operation_failure = Some(format!(
                "Prepared command failed or effects remain unverified: {e}; inspect the exact destination; no automatic cleanup or retry"
            ));
        }
        self.restore_execution(facts, result.is_err());
        result
    }
}
