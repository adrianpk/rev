use super::*;
use crate::git::families::Subtree;
use std::os::unix::ffi::OsStrExt;

impl Runner {
    pub(super) fn family_commit(
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
    pub(super) fn subtree_remote(
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
            check_push_entry(input, entry)?;
        }
        Ok(())
    }
    pub(super) fn prepare_subtree(
        &mut self,
        input: &mut preparation::Preparation,
        destination: &Path,
        source: &Arc<Sequence>,
        admission: &mut Lease,
    ) -> Result<Subtree, String> {
        let family = input.family;
        let head = source
            .refs
            .head
            .ok_or("Subtree requires an existing HEAD commit")?;
        check_subtree_source(source)?;
        let configuration = self.transport_config(admission)?;
        let SubtreeTarget {
            object: target,
            remote,
        } = self.subtree_target(input, head, &configuration)?;
        let tree = self.prepared_subtree_tree(input, target)?;
        check_subtree_prefix(family, destination)?;
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
        plan: &Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let change = SubtreeChange {
            plan,
            state: plan.subtree().expect("subtree plan"),
            source: plan.source.as_ref().expect("subtree source"),
            before: plan
                .source
                .as_ref()
                .expect("subtree source")
                .refs
                .head
                .expect("prepared HEAD"),
        };
        change.run(self, admission)
    }

    fn subtree_target(
        &mut self,
        input: &mut preparation::Preparation,
        head: ObjectId,
        configuration: &[u8],
    ) -> Result<SubtreeTarget, String> {
        let family = input.family;
        if matches!(
            family,
            Family::SubtreeAdd | Family::SubtreePull | Family::SubtreePush
        ) {
            return self.subtree_transport_target(input, head, configuration);
        }
        let args = &mut input.args;
        let object = {
            let object = self.family_commit(&args[5], head.format())?;
            args[5] = object.hex().into();
            if family == Family::SubtreeSplit && object != head {
                // Splitting a retained older commit remains explicit, but
                // native saved sources still bind the current context.
                self.subtree_tree(object, Some(&args[3]))?;
            }
            object
        };
        Ok(SubtreeTarget {
            object,
            remote: None,
        })
    }
    fn subtree_transport_target(
        &mut self,
        input: &mut preparation::Preparation,
        head: ObjectId,
        configuration: &[u8],
    ) -> Result<SubtreeTarget, String> {
        let family = input.family;
        let args = &mut input.args;
        let repository = args[5].clone();
        let reference = args[6].clone();
        if family == Family::SubtreePush {
            self.subtree_push_policy(configuration, &repository)?;
        }
        let object = self.subtree_remote(&repository, &reference, head.format())?;
        let remote = Some((repository, reference, object));
        let object = if family == Family::SubtreePush {
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
        };
        Ok(SubtreeTarget { object, remote })
    }
    fn prepared_subtree_tree(
        &mut self,
        input: &preparation::Preparation,
        target: ObjectId,
    ) -> Result<Option<ObjectId>, String> {
        let family = input.family;
        let args = &input.args;
        let tree = match family {
            Family::SubtreePush | Family::SubtreeSplit => {
                Some(self.subtree_tree(target, Some(&args[3]))?)
            }
            Family::SubtreeMerge => Some(self.subtree_tree(target, None)?),
            _ => None,
        };
        Ok(tree)
    }
}

struct SubtreeTarget {
    object: ObjectId,
    remote: Option<(OsString, OsString, Option<ObjectId>)>,
}

/// A subtree change retains its original branch and prepared family proof.
struct SubtreeChange<'a> {
    plan: &'a Arc<FamilyPlan>,
    state: &'a Subtree,
    source: &'a Arc<Sequence>,
    before: ObjectId,
}
impl SubtreeChange<'_> {
    fn run(&self, runner: &mut Runner, admission: &mut Lease) -> Result<Candidate, String> {
        let plan = self.plan;
        self.check_review(runner, admission)?;
        let split = plan.family == Family::SubtreeSplit;
        runner.private_output = !split;
        let (executed, facts) = runner.execute_change(&plan.args, None);
        runner.private_output = true;
        if runner.retained.is_some() {
            runner.restore_execution(facts, true);
            return Err(
                "Subtree cleanup remains owned; no retry, force or rollback was attempted".into(),
            );
        }
        let result = self.verify(runner, executed, &facts, admission);
        if let Err(e) = &result {
            runner.operation_failure = Some(format!(
                "Subtree result is unverified: {e}; no automatic retry, force, cleanup or rollback"
            ));
        }
        runner.restore_execution(facts, result.is_err());
        runner.private_output = !split || result.is_err();
        result
    }
    fn check_review(&self, runner: &mut Runner, admission: &mut Lease) -> Result<(), String> {
        let plan = self.plan;
        let state = self.state;
        let before = self.before;
        runner.family_relative_destination(&plan.args[3])?;
        if runner.transport_config(admission)? != state.configuration {
            return Err("Subtree configuration changed while review was open".into());
        }
        if let Some((repository, reference, object)) = &state.remote
            && runner.subtree_remote(repository, reference, before.format())? != *object
        {
            return Err("Subtree remote branch changed while review was open".into());
        }
        Ok(())
    }
    fn verify(
        &self,
        runner: &mut Runner,
        executed: Result<(), String>,
        facts: &index::Execution,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let state = self.state;
        let source = self.source;
        let refs = runner.integration_refs(&source.refs.observation, admission)?;
        let actual = runner.read_sequence(&refs, admission)?;
        if actual.family.is_some() {
            return Ok(self.stopped(runner, actual));
        }
        executed?;
        runner.subtree_preserved(source, &actual)?;
        let object = self.result_object(runner, &actual, facts, admission)?;
        if runner.transport_config(admission)? != state.configuration {
            return Err("Subtree changed effective configuration".into());
        }
        self.publish(runner, object, admission)
    }
    fn stopped(&self, runner: &mut Runner, actual: Sequence) -> Candidate {
        let state = self.state;
        let before = self.before;
        runner.in_progress = actual.family == Some(SequenceFamily::Merge)
            && actual.original == Some(before)
            && actual.targets == [state.target];
        runner.operation_failure = Some("Subtree stopped in an actual Git sequence; inspect checked operation state; no automatic continue/abort".into());
        Candidate::Sequence(actual)
    }
    fn result_object(
        &self,
        runner: &mut Runner,
        actual: &Sequence,
        facts: &index::Execution,
        admission: &mut Lease,
    ) -> Result<Option<ObjectId>, String> {
        if self.plan.family == Family::SubtreeSplit {
            self.split_object(runner, actual, facts, admission)
        } else if self.plan.family == Family::SubtreePush {
            self.push_object(runner, actual, admission)
        } else {
            self.merge_object(runner, actual)
        }
    }
    fn split_object(
        &self,
        runner: &mut Runner,
        actual: &Sequence,
        facts: &index::Execution,
        admission: &mut Lease,
    ) -> Result<Option<ObjectId>, String> {
        let state = self.state;
        let before = self.before;
        actual.refs.head.ok_or("Subtree result lost HEAD")?;
        let object = ObjectId::parse(
            facts
                .stdout()
                .strip_suffix(b"\n")
                .ok_or("Subtree split has no full object response")?,
            before.format(),
        )?;
        if runner.subtree_tree(object, None)? != state.tree.ok_or("Split tree unavailable")?
            || !self.local_source_preserved(runner, actual, admission)?
        {
            return Err(
                "Subtree split object/tree or preserved source postcondition differs".into(),
            );
        }
        Ok(Some(object))
    }
    fn push_object(
        &self,
        runner: &mut Runner,
        actual: &Sequence,
        admission: &mut Lease,
    ) -> Result<Option<ObjectId>, String> {
        let state = self.state;
        let before = self.before;
        actual.refs.head.ok_or("Subtree result lost HEAD")?;
        let (repository, reference, _) = state.remote.as_ref().expect("push endpoint");
        let object = runner
            .subtree_remote(repository, reference, before.format())?
            .ok_or("Subtree push destination is absent after execution")?;
        if runner.subtree_tree(object, None)? != state.tree.ok_or("Push tree unavailable")?
            || !self.local_source_preserved(runner, actual, admission)?
        {
            return Err(
                "Subtree push branch/tree or preserved local source postcondition differs".into(),
            );
        }
        Ok(Some(object))
    }
    fn merge_object(
        &self,
        runner: &mut Runner,
        actual: &Sequence,
    ) -> Result<Option<ObjectId>, String> {
        let plan = self.plan;
        let state = self.state;
        let after = actual.refs.head.ok_or("Subtree result lost HEAD")?;
        self.check_ancestry(runner, after)?;
        self.check_prefix_paths(runner, after)?;
        if plan.family == Family::SubtreeAdd
            && runner.subtree_tree(after, Some(&plan.args[3]))?
                != runner.subtree_tree(state.target, None)?
        {
            return Err("Added subtree tree differs from its exact source object".into());
        }
        Ok(Some(after))
    }
    fn local_source_preserved(
        &self,
        runner: &mut Runner,
        actual: &Sequence,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        Ok(actual.refs.head == Some(self.before)
            && actual.index == self.source.index
            && runner.stash_evidence(self.source, admission)? == self.plan.evidence)
    }
    fn check_ancestry(&self, runner: &mut Runner, after: ObjectId) -> Result<(), String> {
        let state = self.state;
        let before = self.before;
        for ancestor in [before, state.target] {
            runner.command(
                &["merge-base", "--is-ancestor", &ancestor.hex(), &after.hex()],
                Phase::Verifying,
                true,
            )?;
            if !runner.exit.is_some_and(|s| s.success()) {
                return Err("Subtree result lacks the exact original/target ancestry".into());
            }
        }
        Ok(())
    }
    fn check_prefix_paths(&self, runner: &mut Runner, after: ObjectId) -> Result<(), String> {
        let plan = self.plan;
        let before = self.before;
        runner.subtree_tree(after, Some(&plan.args[3]))?;
        let outside = OsString::from_vec(
            [
                b":(exclude,literal)".as_slice(),
                plan.args[3].as_encoded_bytes(),
            ]
            .concat(),
        );
        let changed = runner
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
        Ok(())
    }
    fn publish(
        &self,
        runner: &mut Runner,
        object: Option<ObjectId>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let plan = self.plan;
        let source = self.source;
        let (identity, status) = runner.observe(admission)?;
        let mut observation = Observation {
            context: source.refs.observation.context,
            generation: runner.progress.lock().expect("Git progress").operation.0,
            identity,
            status,
            admission: admission.split(0),
        };
        let size = observation.bytes() + std::mem::size_of::<FamilyResult>() + 4096;
        admission.grow(size)?;
        observation.admission = admission.split(observation.bytes());
        runner.effects = Effects::Verified;
        Ok(Candidate::FamilyResult(FamilyResult {
            plan: std::sync::Arc::clone(plan),
            created: None,
            object,
            source: Some(observation),
            _admission: admission.split(4096),
        }))
    }
}

fn check_push_entry(input: &[u8], entry: &[u8]) -> Result<(), String> {
    let Some(split) = entry.iter().position(|b| *b == b'\n') else {
        return Ok(());
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
        return Err(
            "Subtree push endpoint has an active URL rewrite; use an explicit unrevised endpoint"
                .into(),
        );
    }
    Ok(())
}
fn check_subtree_source(source: &Sequence) -> Result<(), String> {
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
    Ok(())
}
fn check_subtree_prefix(family: Family, destination: &Path) -> Result<(), String> {
    if family != Family::SubtreeAdd
        && (!destination.is_dir()
            || std::fs::canonicalize(destination).map_err(|e| e.to_string())? != destination)
    {
        return Err("Subtree prefix is not an existing ordinary canonical directory".into());
    }
    Ok(())
}
