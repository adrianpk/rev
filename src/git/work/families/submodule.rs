use super::*;
use crate::git::families::Submodule;
use std::os::unix::ffi::OsStrExt;

impl Runner {
    fn submodule_key(name: &std::ffi::OsStr, suffix: &[u8]) -> OsString {
        OsString::from_vec([b"submodule.".as_slice(), name.as_bytes(), suffix].concat())
    }

    pub(super) fn submodule_state(
        &mut self,
        family: Family,
        path: &Path,
        source: &Arc<Sequence>,
        admission: &mut Lease,
    ) -> Result<Submodule, String> {
        let configuration = self.command(
            &["config", "--null", "--list", "--show-origin"],
            Phase::Checking,
            false,
        )?;
        admission.grow(configuration.capacity() + 4096)?;
        let mut state = Submodule {
            configuration,
            name: None,
            object: None,
            url: Vec::new(),
            child: None,
            evidence: [Vec::new(), Vec::new(), Vec::new()],
        };
        if family == Family::SubmoduleList {
            return Ok(state);
        }
        let relative = path
            .strip_prefix(&self.directory)
            .map_err(|_| "Submodule path escaped its repository")?;
        self.family_relative_destination(relative.as_os_str())?;
        let entries = self
            .command_os(
                &[
                    "--literal-pathspecs".into(),
                    "ls-files".into(),
                    "--stage".into(),
                    "-z".into(),
                    "--".into(),
                    relative.as_os_str().into(),
                ],
                Phase::Checking,
                false,
                None,
            )?
            .0;
        if !entries.is_empty() {
            let expected_suffix =
                [b"\t".as_slice(), relative.as_os_str().as_bytes(), b"\0"].concat();
            let entry = entries
                .strip_suffix(expected_suffix.as_slice())
                .ok_or("Submodule path does not identify one exact index entry")?;
            let entry = entry
                .strip_prefix(b"160000 ")
                .and_then(|e| e.strip_suffix(b" 0"))
                .ok_or("Selected path is not one unconflicted gitlink")?;
            state.object = Some(ObjectId::parse(
                entry,
                source.refs.observation.identity.format,
            )?);
        }
        if (family == Family::SubmoduleAdd) != state.object.is_none() {
            return Err("Submodule add requires an absent gitlink; other actions require an exact existing gitlink".into());
        }
        let modules = self.directory.join(".gitmodules");
        let exists = match std::fs::symlink_metadata(modules) {
            Ok(m) if m.is_file() && !m.file_type().is_symlink() => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            _ => return Err(".gitmodules is not an ordinary file".into()),
        };
        if exists {
            let paths = self.command(
                &[
                    "config",
                    "--null",
                    "--file=.gitmodules",
                    "--get-regexp",
                    "^submodule\\..*\\.path$",
                ],
                Phase::Checking,
                true,
            )?;
            for field in paths.split(|b| *b == 0).filter(|f| !f.is_empty()) {
                let split = field
                    .iter()
                    .position(|b| *b == b'\n')
                    .ok_or("Malformed submodule path configuration")?;
                if &field[split + 1..] == relative.as_os_str().as_bytes() {
                    let name = field[..split]
                        .strip_prefix(b"submodule.")
                        .and_then(|n| n.strip_suffix(b".path"))
                        .ok_or("Malformed submodule name")?;
                    if state
                        .name
                        .replace(OsString::from_vec(name.into()))
                        .is_some()
                    {
                        return Err("Multiple submodule names map to the selected path".into());
                    }
                }
            }
        }
        if let Some(name) = &state.name {
            state.url = self
                .command_os(
                    &[
                        "config".into(),
                        "--null".into(),
                        "--file=.gitmodules".into(),
                        "--get".into(),
                        Self::submodule_key(name, b".url"),
                    ],
                    Phase::Checking,
                    false,
                    None,
                )?
                .0;
            if state.url.iter().filter(|b| **b == 0).count() != 1 {
                return Err("Submodule requires one explicit URL".into());
            }
        } else if family != Family::SubmoduleAdd {
            return Err("Selected gitlink has no exact .gitmodules path and URL".into());
        }
        if family == Family::SubmoduleAdd && state.name.is_some() {
            return Err("Submodule destination is already declared in .gitmodules".into());
        }
        if path.join(".git").try_exists().map_err(|e| e.to_string())? {
            let directory = self.directory.clone();
            self.directory = path.into();
            let child: Result<Arc<Sequence>, String> = (|| {
                let (identity, status) = self.observe(admission)?;
                if identity.worktree.as_deref() != Some(path) {
                    return Err("Submodule identity differs from its checked path".into());
                }
                let mut observation = Observation {
                    context: source.refs.observation.context,
                    generation: source.id.0,
                    identity,
                    status,
                    admission: admission.split(0),
                };
                admission.grow(observation.bytes())?;
                observation.admission = admission.split(observation.bytes());
                let refs = self.read_refs(Arc::new(observation), admission)?;
                let sequence = Arc::new(self.read_sequence(&Arc::new(refs), admission)?);
                if sequence.family.is_some()
                    || sequence
                        .refs
                        .observation
                        .status
                        .as_ref()
                        .is_none_or(|s| s.completeness != Completeness::Complete)
                {
                    return Err(
                        "Submodule requires complete status and no active child sequence".into(),
                    );
                }
                if matches!(family, Family::SubmoduleUpdate | Family::SubmoduleDeinit)
                    && sequence
                        .refs
                        .observation
                        .status
                        .as_ref()
                        .is_some_and(|s| !s.paths.is_empty())
                {
                    return Err("Submodule update/deinit refuses dirty child state; no force or hidden stash".into());
                }
                state.evidence = self.stash_evidence(&sequence, admission)?;
                Ok(sequence)
            })();
            self.directory = directory;
            state.child = Some(child?);
        } else if path.is_dir()
            && std::fs::read_dir(path)
                .map_err(|e| e.to_string())?
                .next()
                .is_some()
        {
            return Err("Uninitialized submodule destination contains unrelated files".into());
        }
        admission.grow(state.bytes())?;
        Ok(state)
    }

    pub(super) fn execute_submodule(
        &mut self,
        plan: &Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let list = plan.family == Family::SubmoduleList;
        // Configuration probes stay private. Only the explicitly requested
        // status listing may retain its actual output in the process record.
        self.private_output = !list;
        let (executed, facts) = self.execute_change(&plan.args, None);
        self.private_output = true;
        if list {
            self.effects = Effects::NotStarted;
        }
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Submodule cleanup remains owned; no retry or removal was attempted".into(),
            );
        }
        let result = (|| {
            executed?;
            let expected = plan.source.as_ref().expect("submodule source");
            let refs = self.integration_refs(&expected.refs.observation, admission)?;
            let current = Arc::new(self.read_sequence(&refs, admission)?);
            if current.refs.head != expected.refs.head || current.family.is_some() {
                return Err("Submodule command unexpectedly changed HEAD or sequence state".into());
            }
            if plan.family != Family::SubmoduleAdd && current.index != expected.index {
                return Err("Submodule command unexpectedly changed the parent index".into());
            }
            if list {
                self.checked_sequence(expected, admission)?;
                if self.stash_evidence(expected, admission)? != plan.evidence {
                    return Err("Saved repository bytes changed during submodule listing".into());
                }
            } else if plan.family == Family::SubmoduleDeinit {
                if plan.destination.join(".git").exists()
                    || (plan.destination.is_dir()
                        && std::fs::read_dir(&plan.destination)
                            .map_err(|e| e.to_string())?
                            .next()
                            .is_some())
                {
                    return Err(
                        "Deinit did not leave an absent or empty uninitialized destination".into(),
                    );
                }
                let name = plan
                    .submodule()
                    .and_then(|s| s.name.as_ref())
                    .expect("checked name");
                let url = self
                    .command_os(
                        &[
                            "config".into(),
                            "--null".into(),
                            "--get".into(),
                            Self::submodule_key(name, b".url"),
                        ],
                        Phase::Verifying,
                        true,
                        None,
                    )?
                    .0;
                if !url.is_empty() {
                    return Err("Deinit retained the selected submodule URL registration".into());
                }
            } else {
                let actual = self.submodule_state(
                    Family::SubmoduleSync,
                    &plan.destination,
                    &current,
                    admission,
                )?;
                if matches!(plan.family, Family::SubmoduleAdd | Family::SubmoduleUpdate)
                    && actual.child.as_ref().is_none_or(|c| {
                        c.refs.head != actual.object
                            || c.refs.head.is_none()
                            || c.refs
                                .observation
                                .status
                                .as_ref()
                                .is_none_or(|s| !s.paths.is_empty())
                    })
                {
                    return Err(
                        "Submodule checkout does not match its complete exact gitlink object"
                            .into(),
                    );
                }
                if plan.family == Family::SubmoduleAdd {
                    let name = &plan.args[plan.args.len() - 4];
                    if actual.name.as_deref() != Some(name.as_os_str()) {
                        return Err(
                            "Added submodule did not retain its reviewed metadata name".into()
                        );
                    }
                    let url = &plan.args[plan.args.len() - 2];
                    if actual.url.strip_suffix(b"\0") != Some(url.as_encoded_bytes()) {
                        return Err("Added submodule did not retain its reviewed source URL".into());
                    }
                }
                if plan.family == Family::SubmoduleSync {
                    let name = actual.name.as_ref().expect("checked module name");
                    let url = self
                        .command_os(
                            &[
                                "config".into(),
                                "--null".into(),
                                "--get".into(),
                                Self::submodule_key(name, b".url"),
                            ],
                            Phase::Verifying,
                            false,
                            None,
                        )?
                        .0;
                    if url != actual.url {
                        return Err("Sync URL cannot be verified against .gitmodules; inspect resolved relative URLs explicitly".into());
                    }
                }
            }
            let (identity, status) = self.observe(admission)?;
            let mut source = Observation {
                context: expected.refs.observation.context,
                generation: self.progress.lock().expect("Git progress").operation.0,
                identity,
                status,
                admission: admission.split(0),
            };
            let size = source.bytes() + std::mem::size_of::<FamilyResult>() + 4096;
            admission.grow(size)?;
            source.admission = admission.split(source.bytes());
            if !list {
                self.effects = Effects::Verified;
            }
            Ok(Candidate::FamilyResult(FamilyResult {
                plan: std::sync::Arc::clone(plan),
                created: None,
                object: None,
                source: Some(source),
                _admission: admission.split(4096),
            }))
        })();
        if let Err(e) = &result {
            self.operation_failure = Some(format!(
                "Submodule result is unverified: {e}; no automatic retry, force, removal or rollback"
            ));
        }
        self.restore_execution(facts, result.is_err());
        self.private_output = !list || result.is_err();
        result
    }
}
