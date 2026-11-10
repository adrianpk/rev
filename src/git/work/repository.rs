use super::*;
mod worktrees;
use crate::git::repository::{Request, StashAction};
use crate::git::{StashDetail, StashEntry, StashPlan, Stashes};
use std::os::unix::ffi::OsStrExt;

impl Runner {
    pub(super) fn repository(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::Worktrees(request) => self.worktrees(request, admission),
            Request::Stashes(observation) => {
                let refs = self.integration_refs(&observation, admission)?;
                self.read_stashes(std::sync::Arc::clone(&refs.observation), admission)
                    .map(Candidate::Stashes)
            }
            Request::Show { stashes, object } => self.show_stash(stashes, object, admission),
            Request::Prepare {
                stashes, action, ..
            } => self.prepare_stash(stashes, action, admission),
            Request::Stash(plan) => self.change_stash(&plan, admission),
        }
    }
    fn show_stash(
        &mut self,
        stashes: Arc<Stashes>,
        object: ObjectId,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_stash_list(&stashes, admission)?;
        let (bytes, truncated) = self.command_os(
            &[
                "--no-replace-objects".into(),
                "stash".into(),
                "show".into(),
                "--include-untracked".into(),
                "--patch".into(),
                "--binary".into(),
                "--no-ext-diff".into(),
                "--no-textconv".into(),
                object.hex().into(),
            ],
            Phase::Refreshing,
            false,
            Some(self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES)),
        )?;
        self.check_stash_list(&stashes, admission)?;
        let size = bytes.capacity() + std::mem::size_of::<StashDetail>();
        admission.grow(size)?;
        Ok(Candidate::StashDetail(StashDetail {
            stashes,
            object,
            bytes,
            completeness: if truncated {
                Completeness::Truncated {
                    reason: "Stash detail bound",
                }
            } else {
                Completeness::Complete
            },
            _admission: admission.split(size),
        }))
    }
    fn read_stashes(
        &mut self,
        observation: Arc<Observation>,
        admission: &mut Lease,
    ) -> Result<Stashes, String> {
        let bytes = self.command(
            &[
                "stash",
                "list",
                "-z",
                "--format=%H%x00%gd%x00%gs",
                &format!("--max-count={}", self.policy.max_refs + 1),
            ],
            Phase::Refreshing,
            false,
        )?;
        admission.grow(
            bytes.len() * 2
                + self.policy.max_refs.min(bytes.len() / 40) * std::mem::size_of::<StashEntry>()
                + 512,
        )?;
        let mut entries = Vec::new();
        let mut fields = bytes
            .split_inclusive(|b| *b == 0)
            .map(|field| field.strip_suffix(b"\0").unwrap_or(field));
        let mut completeness = Completeness::Complete;
        while let Some(object) = fields.next() {
            let selector = fields.next().ok_or("Stash record lacks selector")?;
            let subject = fields.next().ok_or("Stash record lacks subject")?;
            if entries.len() == self.policy.max_refs {
                completeness = Completeness::Truncated {
                    reason: "Stash count bound",
                };
                break;
            }
            check_stash_fields(selector, subject)?;
            entries.push(StashEntry {
                object: ObjectId::parse(object, observation.identity.format)?,
                selector: selector.into(),
                subject: subject.into(),
            });
        }
        let id = self.progress.lock().expect("Git progress owner").operation;
        let size = std::mem::size_of::<Stashes>()
            + entries.capacity() * std::mem::size_of::<StashEntry>()
            + entries
                .iter()
                .map(|e| e.selector.capacity() + e.subject.capacity())
                .sum::<usize>();
        admission.grow(size)?;
        Ok(Stashes {
            id,
            observation,
            entries,
            completeness,
            _admission: admission.split(size),
        })
    }
    fn check_stash_list(
        &mut self,
        expected: &Stashes,
        admission: &mut Lease,
    ) -> Result<(), String> {
        let refs = self.integration_refs(&expected.observation, admission)?;
        let actual = self.read_stashes(std::sync::Arc::clone(&refs.observation), admission)?;
        if !expected.same(&actual) {
            return Err(
                "Stash list changed; refresh the exact object selection before acting".into(),
            );
        }
        Ok(())
    }
    fn prepare_stash(
        &mut self,
        stashes: Arc<Stashes>,
        action: StashAction,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_stash_list(&stashes, admission)?;
        let refs = self.integration_refs(&stashes.observation, admission)?;
        let source = self.read_sequence(&refs, admission)?;
        StashPreparation {
            stashes,
            action,
            source,
            paths: Vec::new(),
            base: None,
        }
        .prepare(self, admission)
    }
    fn stash_paths(
        &self,
        bytes: &[u8],
        paths: &mut Vec<PathBuf>,
        admission: &mut Lease,
    ) -> Result<(), String> {
        if bytes.len() > self.policy.max_path_bytes {
            return Err("Stash paths exceed complete byte admission".into());
        }
        admission.grow(
            bytes.len() * 2
                + bytes.iter().filter(|b| **b == 0).count() * std::mem::size_of::<PathBuf>(),
        )?;
        for path in bytes
            .split_inclusive(|b| *b == 0)
            .map(|field| field.strip_suffix(b"\0").unwrap_or(field))
        {
            if paths.len() == self.policy.max_status_paths || path.is_empty() || path.len() > 4096 {
                return Err("Stash paths exceed complete count/path admission".into());
            }
            let path = PathBuf::from(OsString::from_vec(path.into()));
            if !path
                .components()
                .all(|p| matches!(p, std::path::Component::Normal(_)))
            {
                return Err("Stash contains a non-relative canonical path".into());
            }
            paths.push(path);
        }
        Ok(())
    }
    fn change_stash(
        &mut self,
        plan: &Arc<StashPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_stash_launch(plan, admission)?;
        let selector = plan
            .action
            .object()
            .and_then(|o| plan.stashes.entry(o))
            .map(|e| OsString::from_vec(e.selector.clone()));
        let args = stash_arguments(plan, selector.as_ref())?;
        let (executed, facts) = self.execute_change(&args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Stash cleanup remains owned; inspect actual state without retry or rollback"
                    .into(),
            );
        }
        let mut change = StashChange {
            plan,
            selector,
            executed,
            facts,
        };
        let result = change.observe(self, admission);
        self.restore_execution(change.facts, change.executed.is_err());
        if result.is_err() {
            self.operation_failure = Some(
                "Stash effects remain unverified; inspect actual repository state without replay"
                    .into(),
            );
        }
        result
    }
    fn check_stash_launch(
        &mut self,
        plan: &StashPlan,
        admission: &mut Lease,
    ) -> Result<(), String> {
        self.checked_sequence(&plan.source, admission)?;
        self.check_stash_list(&plan.stashes, admission)?;
        if plan.evidence != self.stash_evidence(&plan.source, admission)? {
            return Err(
                "Saved index, working files or untracked bytes changed while stash review was open"
                    .into(),
            );
        }
        Ok(())
    }
    fn stash_refs_match(plan: &StashPlan, sequence: &Sequence) -> bool {
        if let StashAction::Branch { name, .. } = &plan.action {
            let target = format!("refs/heads/{name}");
            sequence.refs.head == plan.base
                && sequence.refs.head_ref == target.as_bytes()
                && sequence.refs.entries.len() == plan.source.refs.entries.len() + 1
                && plan
                    .source
                    .refs
                    .entries
                    .iter()
                    .all(|r| sequence.refs.reference(&r.name) == Some(r))
                && sequence
                    .refs
                    .reference(target.as_bytes())
                    .is_some_and(|r| Some(r.object) == plan.base)
        } else {
            sequence.refs.same(&plan.source.refs)
        }
    }
    pub(super) fn stash_evidence(
        &mut self,
        source: &Sequence,
        admission: &mut Lease,
    ) -> Result<[Vec<u8>; 3], String> {
        let index = self.command(&["ls-files", "--stage", "-z"], Phase::Checking, false)?;
        // An unborn repository has no HEAD baseline. Exact staged entries
        // above plus the working diff from that index still cover saved state.
        let mut args = vec![
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--binary",
        ];
        if source.refs.head.is_some() {
            args.push("HEAD");
        }
        args.push("--");
        let patch = self.command(&args, Phase::Checking, false)?;
        let mut untracked = Vec::new();
        if let Some(status) = &source.refs.observation.status {
            for entry in status.group(Group::Untracked) {
                let bytes = self
                    .command_os(
                        &[
                            "hash-object".into(),
                            "--no-filters".into(),
                            "--".into(),
                            entry.path.as_os_str().into(),
                        ],
                        Phase::Checking,
                        false,
                        None,
                    )?
                    .0;
                let size = entry.path.as_os_str().len() + bytes.len() + 1;
                admission.grow(size)?;
                untracked.extend_from_slice(entry.path.as_os_str().as_bytes());
                untracked.push(0);
                untracked.extend(bytes);
            }
        }
        admission.grow(index.capacity() + patch.capacity())?;
        Ok([index, patch, untracked])
    }
}

fn check_stash_fields(selector: &[u8], subject: &[u8]) -> Result<(), String> {
    if selector.len() < 9
        || selector.len() > 4096
        || subject.len() > 4096
        || !selector.starts_with(b"stash@{")
        || !selector.ends_with(b"}")
        || !selector[7..selector.len() - 1]
            .iter()
            .all(u8::is_ascii_digit)
    {
        return Err("Stash metadata has an unsupported selector or field bound".into());
    }
    Ok(())
}

/// Preparation retains the checked source and action while discovering exact affected paths.
struct StashPreparation {
    stashes: Arc<Stashes>,
    action: StashAction,
    source: Sequence,
    paths: Vec<PathBuf>,
    base: Option<ObjectId>,
}
impl StashPreparation {
    fn prepare(mut self, runner: &mut Runner, admission: &mut Lease) -> Result<Candidate, String> {
        self.check_source()?;
        match self.action {
            StashAction::Create { .. } => self.saved_paths(runner, admission)?,
            StashAction::Drop(_) => {}
            _ => self.restore_paths(runner, admission)?,
        }
        self.finish(runner, admission)
    }
    fn check_source(&self) -> Result<(), String> {
        let status = self
            .source
            .refs
            .observation
            .status
            .as_ref()
            .ok_or("Stashes require a working tree")?;
        if self.source.refs.completeness != Completeness::Complete
            || self.source.family.is_some()
            || !self.source.conflicts.is_empty()
            || status.completeness != Completeness::Complete
        {
            return Err(
                "Stash actions require complete state without an active sequence or unmerged index"
                    .into(),
            );
        }
        self.source
            .refs
            .head
            .ok_or("Stash actions require an existing HEAD")?;
        Ok(())
    }
    fn status(&self) -> Result<&Status, String> {
        self.source
            .refs
            .observation
            .status
            .as_ref()
            .ok_or_else(|| "Stashes require a working tree".into())
    }
    fn saved_paths(&mut self, runner: &Runner, admission: &mut Lease) -> Result<(), String> {
        let status = self
            .source
            .refs
            .observation
            .status
            .as_ref()
            .ok_or_else(|| "Stashes require a working tree".to_string())?;
        if self.stashes.entries.len() == runner.policy.max_refs {
            return Err("New stash would exceed complete list admission".into());
        }
        if status.paths.is_empty() {
            return Err("No saved tracked or untracked changes to stash".into());
        }
        admission.grow(
            status.paths.len() * 2 * std::mem::size_of::<PathBuf>()
                + status
                    .paths
                    .iter()
                    .map(|p| p.path.capacity() + p.old_path.as_ref().map_or(0, PathBuf::capacity))
                    .sum::<usize>(),
        )?;
        self.paths.extend(
            status
                .paths
                .iter()
                .flat_map(|p| std::iter::once(p.path.clone()).chain(p.old_path.clone())),
        );
        Ok(())
    }
    fn parents(&self, runner: &mut Runner) -> Result<(ObjectId, Vec<ObjectId>), String> {
        let action = &self.action;
        let source = &self.source;
        let object = action.object().ok_or("Exact stash target missing")?;
        let bytes = runner.command(
            &[
                "--no-replace-objects",
                "show",
                "--no-patch",
                "--format=%P",
                &object.hex(),
            ],
            Phase::Checking,
            false,
        )?;
        let parents: Vec<_> = bytes
            .strip_suffix(b"\n")
            .unwrap_or(&bytes)
            .split(|b| *b == b' ')
            .take(4)
            .map(|p| ObjectId::parse(p, source.refs.observation.identity.format))
            .collect::<Result<_, _>>()?;
        if !(2..=3).contains(&parents.len()) {
            return Err("Selected object is not a supported two/three-parent stash".into());
        }
        Ok((object, parents))
    }
    fn restore_paths(&mut self, runner: &mut Runner, admission: &mut Lease) -> Result<(), String> {
        if !self.status()?.paths.is_empty() {
            return Err("Stash restoration requires a complete clean index and working tree; no implicit save or stash".into());
        }
        let (object, parents) = self.parents(runner)?;
        self.base = Some(parents[0]);
        let changed = runner.command(
            &[
                "--no-replace-objects",
                "diff-tree",
                "--no-commit-id",
                "--no-renames",
                "--name-only",
                "-r",
                "-z",
                &parents[0].hex(),
                &object.hex(),
                "--",
            ],
            Phase::Checking,
            false,
        )?;
        runner.stash_paths(&changed, &mut self.paths, admission)?;
        if let Some(untracked) = parents.get(2) {
            let bytes = runner.command(
                &[
                    "--no-replace-objects",
                    "ls-tree",
                    "-r",
                    "--name-only",
                    "-z",
                    &untracked.hex(),
                    "--",
                ],
                Phase::Checking,
                false,
            )?;
            runner.stash_paths(&bytes, &mut self.paths, admission)?;
        }
        self.check_branch(runner)
    }
    fn check_branch(&self, runner: &mut Runner) -> Result<(), String> {
        if let StashAction::Branch { name, .. } = &self.action {
            runner.command(
                &["check-ref-format", "--branch", name],
                Phase::Checking,
                false,
            )?;
            if self
                .source
                .refs
                .reference(format!("refs/heads/{name}").as_bytes())
                .is_some()
            {
                return Err("Stash branch already exists".into());
            }
        }
        Ok(())
    }
    fn finish(mut self, runner: &mut Runner, admission: &mut Lease) -> Result<Candidate, String> {
        self.paths.sort();
        self.paths.dedup();
        if self.paths.len() > runner.policy.max_status_paths {
            return Err("Stash affected paths exceed complete count admission".into());
        }
        let evidence = runner.stash_evidence(&self.source, admission)?;
        let size = std::mem::size_of::<StashPlan>()
            + self.paths.capacity() * std::mem::size_of::<PathBuf>()
            + self.paths.iter().map(PathBuf::capacity).sum::<usize>()
            + evidence.iter().map(Vec::capacity).sum::<usize>()
            + 8192;
        admission.grow(size)?;
        runner.checked_sequence(&self.source, admission)?;
        runner.check_stash_list(&self.stashes, admission)?;
        if evidence != runner.stash_evidence(&self.source, admission)? {
            return Err("Saved stash sources changed during preparation".into());
        }
        Ok(Candidate::StashPlan(StashPlan {
            stashes: self.stashes,
            source: Arc::new(self.source),
            action: self.action,
            paths: self.paths,
            base: self.base,
            evidence,
            _admission: admission.split(size),
        }))
    }
}

fn stash_arguments(plan: &StashPlan, selector: Option<&OsString>) -> Result<Vec<OsString>, String> {
    Ok(match &plan.action {
        StashAction::Create { message } => vec![
            "stash".into(),
            "push".into(),
            "--include-untracked".into(),
            "--message".into(),
            message.into(),
            "--".into(),
        ],
        StashAction::Apply(o) | StashAction::Pop(o) => vec![
            "--no-replace-objects".into(),
            "stash".into(),
            "apply".into(),
            o.hex().into(),
        ],
        StashAction::Branch { object, name } => vec![
            "--no-replace-objects".into(),
            "stash".into(),
            "branch".into(),
            name.into(),
            object.hex().into(),
        ],
        StashAction::Drop(_) => vec![
            "stash".into(),
            "drop".into(),
            selector.cloned().ok_or("Exact stash selector missing")?,
        ],
    })
}
/// The actual changing child owns its result and diagnostic facts through a checked optional drop.
struct StashChange<'a> {
    plan: &'a StashPlan,
    selector: Option<OsString>,
    executed: Result<(), String>,
    facts: index::Execution,
}
impl StashChange<'_> {
    fn observe(&mut self, runner: &mut Runner, admission: &mut Lease) -> Result<Candidate, String> {
        let restored_evidence = self.drop_restored(runner, admission)?;
        let refs = runner.integration_refs(&self.plan.source.refs.observation, admission)?;
        let sequence = runner.read_sequence(&refs, admission)?;
        let stashes =
            runner.read_stashes(std::sync::Arc::clone(&sequence.refs.observation), admission)?;
        ObservedStash { stashes, sequence }.finish(runner, self, &restored_evidence, admission)
    }
    fn drop_restored(
        &mut self,
        runner: &mut Runner,
        admission: &mut Lease,
    ) -> Result<Option<[Vec<u8>; 3]>, String> {
        if !(self.executed.is_ok()
            && matches!(
                self.plan.action,
                StashAction::Pop(_) | StashAction::Branch { .. }
            ))
        {
            return Ok(None);
        }
        let plan = self.plan;
        let restored = self.restored(runner, admission)?;
        let evidence = runner.stash_evidence(&restored, admission)?;
        runner.check_stash_list(&plan.stashes, admission)?;
        let args = [
            "stash".into(),
            "drop".into(),
            self.selector.take().ok_or("Exact stash selector missing")?,
        ];
        (self.executed, self.facts) = runner.execute_change(&args, None);
        if runner.retained.is_some() {
            return Err("Stash removal cleanup remains owned after restoration".into());
        }
        Ok(Some(evidence))
    }
    fn restored(&self, runner: &mut Runner, admission: &mut Lease) -> Result<Sequence, String> {
        let plan = self.plan;
        let refs = runner.integration_refs(&plan.source.refs.observation, admission)?;
        let restored = runner.read_sequence(&refs, admission)?;
        if !restored_stash_admitted(plan, &restored) {
            return Err(
                "Restored stash state changed or is incomplete; its entry was retained".into(),
            );
        }
        Ok(restored)
    }
    fn list_matches(&self, runner: &mut Runner, after: &Stashes) -> Result<bool, String> {
        let plan = self.plan;
        let objects = |s: &Stashes| s.entries.iter().map(|e| e.object).collect::<Vec<_>>();
        let before_objects = objects(&plan.stashes);
        let actual_objects = objects(after);
        let dropped = matches!(
            plan.action,
            StashAction::Pop(_) | StashAction::Branch { .. } | StashAction::Drop(_)
        );
        Ok(if matches!(plan.action, StashAction::Create { .. }) {
            self.created_list(runner, &actual_objects, &before_objects)?
        } else if self.executed.is_ok() && dropped {
            self.dropped_list(&actual_objects, before_objects)?
        } else {
            after.same(&plan.stashes)
        })
    }
    fn created_list(
        &self,
        runner: &mut Runner,
        actual_objects: &[ObjectId],
        before_objects: &[ObjectId],
    ) -> Result<bool, String> {
        let plan = self.plan;
        Ok(actual_objects.len() == before_objects.len() + 1
            && &actual_objects[1..] == before_objects
            && runner.command(
                &[
                    "--no-replace-objects",
                    "rev-parse",
                    &format!("{}^1", actual_objects[0].hex()),
                ],
                Phase::Verifying,
                false,
            )? == format!(
                "{}\n",
                plan.source
                    .refs
                    .head
                    .ok_or("Stash source HEAD missing")?
                    .hex()
            )
            .as_bytes())
    }
    fn dropped_list(
        &self,
        actual_objects: &[ObjectId],
        before_objects: Vec<ObjectId>,
    ) -> Result<bool, String> {
        let plan = self.plan;
        let target = plan.action.object().ok_or("Stash removal target missing")?;
        let mut expected = before_objects;
        expected.remove(
            expected
                .iter()
                .position(|o| *o == target)
                .ok_or("Stash removal target expired")?,
        );
        Ok(actual_objects == expected.as_slice())
    }
    fn state_matches(
        &self,
        runner: &mut Runner,
        sequence: &Sequence,
        restored_evidence: &Option<[Vec<u8>; 3]>,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let plan = self.plan;
        Ok(match plan.action {
            StashAction::Create { .. } => sequence
                .refs
                .observation
                .status
                .as_ref()
                .is_some_and(|s| s.paths.is_empty()),
            StashAction::Drop(_) => {
                sequence.same(&plan.source)
                    && runner.stash_evidence(sequence, admission)? == plan.evidence
            }
            StashAction::Pop(_) | StashAction::Branch { .. } => {
                restored_evidence.as_ref() == Some(&runner.stash_evidence(sequence, admission)?)
            }
            _ => true,
        })
    }
}
fn restored_stash_admitted(plan: &StashPlan, restored: &Sequence) -> bool {
    !(!Runner::stash_refs_match(plan, restored)
        || restored.family.is_some()
        || !restored.conflicts.is_empty()
        || restored
            .refs
            .observation
            .status
            .as_ref()
            .is_none_or(|s| s.completeness != Completeness::Complete))
}
struct ObservedStash {
    stashes: Stashes,
    sequence: Sequence,
}
impl ObservedStash {
    fn finish(
        self,
        runner: &mut Runner,
        change: &StashChange<'_>,
        restored_evidence: &Option<[Vec<u8>; 3]>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let plan = change.plan;
        let sequence = &self.sequence;
        let after = &self.stashes;
        let executed = &change.executed;
        let list_ok = change.list_matches(runner, after)?;
        let refs_ok = Runner::stash_refs_match(plan, sequence);
        let complete = self.complete();
        let admitted = list_ok && refs_ok && complete;
        if !sequence.conflicts.is_empty() && admitted {
            runner.in_progress = true;
            runner.operation_failure = Some("Exact stash restoration stopped with actual unmerged paths; the stash is retained and manual conflict resolution is required".into());
        } else if executed.is_ok() && admitted && sequence.family.is_none() {
            self.verify_state(runner, change, restored_evidence, admission)?;
        } else {
            runner.operation_failure = Some("Stash action failed or exact refs/list/state verification differs; no automatic retry or rollback".into());
        }
        Ok(Candidate::StashResult {
            stashes: self.stashes,
            sequence: self.sequence,
        })
    }
    fn complete(&self) -> bool {
        let after = &self.stashes;
        let sequence = &self.sequence;
        after.completeness == Completeness::Complete
            && sequence
                .refs
                .observation
                .status
                .as_ref()
                .is_some_and(|s| s.completeness == Completeness::Complete)
    }
    fn verify_state(
        &self,
        runner: &mut Runner,
        change: &StashChange<'_>,
        restored_evidence: &Option<[Vec<u8>; 3]>,
        admission: &mut Lease,
    ) -> Result<(), String> {
        let state_ok =
            change.state_matches(runner, &self.sequence, restored_evidence, admission)?;
        if state_ok {
            runner.effects = Effects::Verified;
        } else {
            runner.operation_failure = Some("Stash command completed but saved state postconditions differ; inspect actual state".into());
        }
        Ok(())
    }
}
