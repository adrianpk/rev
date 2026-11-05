use super::*;
use crate::git::families::Subtree;
use std::os::unix::ffi::OsStrExt;

impl Runner {
    fn family_commit(
        &mut self,
        name: &std::ffi::OsStr,
        format: ObjectFormat,
    ) -> Result<ObjectId, String> {
        let commit = OsString::from_vec([name.as_bytes(), b"^{commit}"].concat());
        self.family_object(&commit, format)
    }
    pub(super) fn family_object(
        &mut self,
        expression: &std::ffi::OsStr,
        format: ObjectFormat,
    ) -> Result<ObjectId, String> {
        let bytes = self
            .command_os(
                &[
                    "rev-parse".into(),
                    "--verify".into(),
                    "--end-of-options".into(),
                    expression.into(),
                ],
                Phase::Checking,
                false,
                None,
            )?
            .0;
        ObjectId::parse(
            bytes
                .strip_suffix(b"\n")
                .ok_or("Prepared object response is incomplete")?,
            format,
        )
    }
    fn subtree_tree(
        &mut self,
        commit: ObjectId,
        prefix: Option<&std::ffi::OsStr>,
    ) -> Result<ObjectId, String> {
        let expression = if let Some(prefix) = prefix {
            OsString::from_vec([commit.hex().as_bytes(), b":", prefix.as_bytes()].concat())
        } else {
            format!("{}^{{tree}}", commit.hex()).into()
        };
        let tree = self.family_object(&expression, commit.format())?;
        if self.command(&["cat-file", "-t", &tree.hex()], Phase::Checking, false)? != b"tree\n" {
            return Err("Subtree prefix does not identify an actual tree".into());
        }
        Ok(tree)
    }
    fn subtree_remote(
        &mut self,
        repository: &std::ffi::OsStr,
        reference: &std::ffi::OsStr,
        format: ObjectFormat,
    ) -> Result<Option<ObjectId>, String> {
        if !reference.as_bytes().starts_with(b"refs/heads/") {
            return Err("Subtree transport requires one explicit full refs/heads branch".into());
        }
        self.command_os(
            &["check-ref-format".into(), reference.into()],
            Phase::Checking,
            false,
            None,
        )?;
        let bytes = self
            .command_os(
                &[
                    "ls-remote".into(),
                    "--refs".into(),
                    "--".into(),
                    repository.into(),
                    reference.into(),
                ],
                Phase::Checking,
                false,
                None,
            )?
            .0;
        if bytes.is_empty() {
            return Ok(None);
        }
        let suffix = [b"\t".as_slice(), reference.as_bytes(), b"\n"].concat();
        let object = bytes
            .strip_suffix(suffix.as_slice())
            .ok_or("Subtree remote response differs from its exact branch")?;
        ObjectId::parse(object, format).map(Some)
    }
    fn subtree_push_policy(
        &mut self,
        config: &[u8],
        repository: &std::ffi::OsStr,
    ) -> Result<(), String> {
        let input = repository.as_bytes();
        if !Path::new(repository).is_absolute()
            && !input.windows(3).any(|w| w == b"://")
            && !input.contains(&b':')
        {
            return Err("Subtree push requires an explicit repository URL or absolute path, not a configured remote name".into());
        }
        if self.transport_bool(b"push.followtags".to_vec())? {
            return Err("Subtree push refuses implicit followTags publication".into());
        }
        for entry in config.split(|b| *b == 0).filter(|r| !r.is_empty()) {
            let Some(split) = entry.iter().position(|b| *b == b'\n') else {
                continue;
            };
            let (key, value) = (&entry[..split], &entry[split + 1..]);
            let configured = [b"remote.".as_slice(), input, b"."].concat();
            if key.starts_with(&configured) {
                return Err("Subtree push endpoint is also a configured remote name; use an unambiguous explicit endpoint".into());
            }
            if key == b"push.recursesubmodules" && ![b"no".as_slice(), b"check"].contains(&value) {
                return Err("Subtree push refuses implicit recursive submodule publication".into());
            }
            if key.starts_with(b"url.")
                && (key.ends_with(b".insteadof") || key.ends_with(b".pushinsteadof"))
                && input.starts_with(value)
            {
                return Err("Subtree push endpoint has an active URL rewrite; use an explicit unrevised endpoint".into());
            }
        }
        Ok(())
    }
    pub(super) fn prepare_subtree(
        &mut self,
        family: Family,
        args: &mut [OsString],
        destination: &Path,
        source: &Arc<Sequence>,
        admission: &mut Lease,
    ) -> Result<Subtree, String> {
        let head = source
            .refs
            .head
            .ok_or("Subtree requires an existing HEAD commit")?;
        if source.family.is_some()
            || source.refs.completeness != Completeness::Complete
            || source
                .refs
                .observation
                .status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete || !s.paths.is_empty())
        {
            return Err("Subtree preparation requires complete refs, clean saved state and no active sequence; no implicit stash".into());
        }
        let configuration = self.transport_config(admission)?;
        let mut remote = None;
        let target = match family {
            Family::SubtreeAdd | Family::SubtreePull | Family::SubtreePush => {
                let repository = args[5].clone();
                let reference = args[6].clone();
                if family == Family::SubtreePush {
                    self.subtree_push_policy(&configuration, &repository)?;
                }
                let object = self.subtree_remote(&repository, &reference, head.format())?;
                remote = Some((repository, reference, object));
                if family == Family::SubtreePush {
                    // Git subtree itself prepends refs/heads/ to its operand.
                    // Review retains the full destination, while this exact
                    // argv uses the corresponding non-forced short operand.
                    args[6] = OsString::from_vec(
                        args[6]
                            .as_bytes()
                            .strip_prefix(b"refs/heads/")
                            .expect("checked branch")
                            .into(),
                    );
                    head
                } else {
                    let object = object.ok_or("Subtree source branch is absent")?;
                    args[6] = object.hex().into();
                    object
                }
            }
            _ => {
                let object = self.family_commit(&args[5], head.format())?;
                args[5] = object.hex().into();
                if family == Family::SubtreeSplit && object != head {
                    // Splitting a retained older commit remains explicit, but
                    // native saved sources still bind the current context.
                    self.subtree_tree(object, Some(&args[3]))?;
                }
                object
            }
        };
        let tree = match family {
            Family::SubtreePush | Family::SubtreeSplit => {
                Some(self.subtree_tree(target, Some(&args[3]))?)
            }
            Family::SubtreeMerge => Some(self.subtree_tree(target, None)?),
            _ => None,
        };
        if family != Family::SubtreeAdd
            && (!destination.is_dir()
                || std::fs::canonicalize(destination).map_err(|e| e.to_string())? != destination)
        {
            return Err("Subtree prefix is not an existing ordinary canonical directory".into());
        }
        Ok(Subtree {
            configuration,
            target,
            tree,
            remote,
        })
    }
    fn subtree_preserved(&mut self, expected: &Sequence, actual: &Sequence) -> Result<(), String> {
        if actual.refs.head_ref != expected.refs.head_ref
            || actual.refs.completeness != Completeness::Complete
            || actual
                .refs
                .observation
                .status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete || !s.paths.is_empty())
        {
            return Err(
                "Subtree result lost its exact branch or complete clean saved state".into(),
            );
        }
        if expected
            .refs
            .entries
            .iter()
            .filter(|r| r.name != expected.refs.head_ref)
            .any(|r| {
                actual
                    .refs
                    .reference(&r.name)
                    .is_none_or(|a| a.object != r.object || a.symbolic != r.symbolic)
            })
            || actual.refs.entries.len() != expected.refs.entries.len()
        {
            return Err(
                "Subtree changed another local ref; inspect actual repository facts".into(),
            );
        }
        Ok(())
    }
    pub(super) fn execute_subtree(
        &mut self,
        plan: Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let state = plan.subtree().expect("subtree plan");
        let source = plan.source.as_ref().expect("subtree source");
        let before = source.refs.head.expect("prepared HEAD");
        self.family_relative_destination(&plan.args[3])?;
        if self.transport_config(admission)? != state.configuration {
            return Err("Subtree configuration changed while review was open".into());
        }
        if let Some((repository, reference, object)) = &state.remote
            && self.subtree_remote(repository, reference, before.format())? != *object
        {
            return Err("Subtree remote branch changed while review was open".into());
        }
        let split = plan.family == Family::SubtreeSplit;
        self.private_output = !split;
        let (executed, facts) = self.execute_change(&plan.args, None);
        self.private_output = true;
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Subtree cleanup remains owned; no retry, force or rollback was attempted".into(),
            );
        }
        let result = (|| {
            let refs = self.integration_refs(&source.refs.observation, admission)?;
            let actual = self.read_sequence(refs, admission)?;
            if actual.family.is_some() {
                self.in_progress = actual.family == Some(SequenceFamily::Merge)
                    && actual.original == Some(before)
                    && actual.targets == [state.target];
                self.operation_failure = Some("Subtree stopped in an actual Git sequence; inspect checked operation state; no automatic continue/abort".into());
                return Ok(Candidate::Sequence(actual));
            }
            executed?;
            self.subtree_preserved(source, &actual)?;
            let after = actual.refs.head.ok_or("Subtree result lost HEAD")?;
            let object = if split {
                let object = ObjectId::parse(
                    facts
                        .stdout()
                        .strip_suffix(b"\n")
                        .ok_or("Subtree split has no full object response")?,
                    before.format(),
                )?;
                if self.subtree_tree(object, None)? != state.tree.ok_or("Split tree unavailable")?
                    || after != before
                    || actual.index != source.index
                    || self.stash_evidence(source, admission)? != plan.evidence
                {
                    return Err(
                        "Subtree split object/tree or preserved source postcondition differs"
                            .into(),
                    );
                }
                Some(object)
            } else if plan.family == Family::SubtreePush {
                let (repository, reference, _) = state.remote.as_ref().expect("push endpoint");
                let object = self
                    .subtree_remote(repository, reference, before.format())?
                    .ok_or("Subtree push destination is absent after execution")?;
                if self.subtree_tree(object, None)? != state.tree.ok_or("Push tree unavailable")?
                    || after != before
                    || actual.index != source.index
                    || self.stash_evidence(source, admission)? != plan.evidence
                {
                    return Err(
                        "Subtree push branch/tree or preserved local source postcondition differs"
                            .into(),
                    );
                }
                Some(object)
            } else {
                for ancestor in [before, state.target] {
                    self.command(
                        &["merge-base", "--is-ancestor", &ancestor.hex(), &after.hex()],
                        Phase::Verifying,
                        true,
                    )?;
                    if !self.exit.is_some_and(|s| s.success()) {
                        return Err(
                            "Subtree result lacks the exact original/target ancestry".into()
                        );
                    }
                }
                self.subtree_tree(after, Some(&plan.args[3]))?;
                let outside = OsString::from_vec(
                    [
                        b":(exclude,literal)".as_slice(),
                        plan.args[3].as_encoded_bytes(),
                    ]
                    .concat(),
                );
                let changed = self
                    .command_os(
                        &[
                            "diff".into(),
                            "--no-ext-diff".into(),
                            "--no-textconv".into(),
                            "--name-only".into(),
                            "-z".into(),
                            before.hex().into(),
                            after.hex().into(),
                            "--".into(),
                            ".".into(),
                            outside,
                        ],
                        Phase::Verifying,
                        false,
                        None,
                    )?
                    .0;
                if !changed.is_empty() {
                    return Err("Subtree changed saved paths outside its reviewed prefix".into());
                }
                if plan.family == Family::SubtreeAdd
                    && self.subtree_tree(after, Some(&plan.args[3]))?
                        != self.subtree_tree(state.target, None)?
                {
                    return Err("Added subtree tree differs from its exact source object".into());
                }
                Some(after)
            };
            if self.transport_config(admission)? != state.configuration {
                return Err("Subtree changed effective configuration".into());
            }
            let (identity, status) = self.observe(admission)?;
            let mut observation = Observation {
                context: source.refs.observation.context,
                generation: self.progress.lock().expect("Git progress").operation.0,
                identity,
                status,
                admission: admission.split(0),
            };
            let size = observation.bytes() + std::mem::size_of::<FamilyResult>() + 4096;
            admission.grow(size)?;
            observation.admission = admission.split(observation.bytes());
            self.effects = Effects::Verified;
            Ok(Candidate::FamilyResult(FamilyResult {
                plan: plan.clone(),
                created: None,
                object,
                source: Some(observation),
                _admission: admission.split(4096),
            }))
        })();
        if let Err(e) = &result {
            self.operation_failure = Some(format!(
                "Subtree result is unverified: {e}; no automatic retry, force, cleanup or rollback"
            ));
        }
        self.restore_execution(facts, result.is_err());
        self.private_output = !split || result.is_err();
        result
    }
}
