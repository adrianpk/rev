use super::*;
use crate::git::sequences::Request;
mod control;
mod preparation;

impl Runner {
    pub(super) fn application(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::Prepare {
                history,
                kind,
                choices,
                ..
            } => self
                .prepare_application(history, kind, &choices, admission)
                .map(Candidate::ApplicationPlan),
            Request::Start(plan) => self.start_application(plan, admission),
        }
    }
    pub(super) fn bind_application(
        &mut self,
        sequence: &mut Sequence,
        progress: Arc<ApplyProgress>,
    ) -> Result<(), String> {
        let Some(step) = progress.plan.steps.get(progress.next) else {
            return Err("Retained ordered revision position is invalid".into());
        };
        if sequence.family != Some(progress.plan.kind.family())
            || sequence.current != Some(step.object)
            || sequence.refs.head != Some(progress.before)
            || sequence.refs.head_ref != progress.plan.refs.head_ref
        {
            return Err(
                "Git sequence target, branch or HEAD differs from the retained ordered request"
                    .into(),
            );
        }
        sequence
            ._admission
            .grow(progress.plan.refs.head_ref.len())?;
        sequence.branch = progress.plan.refs.head_ref.clone();
        sequence.original = progress.plan.refs.head;
        sequence.step = Some(progress.next + 1);
        sequence.total = Some(progress.plan.steps.len());
        sequence.application = Some(progress);
        Ok(())
    }
    fn start_application(
        &mut self,
        plan: Arc<ApplyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.verify_ancestry(&plan.history)?;
        let refs = self.check_integration_source(&plan.refs, admission)?;
        if self.read_sequence(&refs, admission)?.family.is_some() {
            return Err(
                "An actual Git sequence is already active; inspect it before applying revisions"
                    .into(),
            );
        }
        let mut completed = Vec::with_capacity(plan.steps.len());
        admission.grow(
            std::mem::size_of::<ApplyProgress>()
                + completed.capacity() * std::mem::size_of::<ObjectId>(),
        )?;
        let before = refs
            .head
            .ok_or("Ordered revisions require an existing HEAD")?;
        let progress = ApplyProgress {
            plan,
            next: 0,
            before,
            completed: std::mem::take(&mut completed),
            _admission: admission.split(0),
        };
        self.apply_steps(progress, refs, admission, None)
    }
    pub(super) fn apply_steps(
        &mut self,
        progress: ApplyProgress,
        expected: Arc<Refs>,
        admission: &mut Lease,
        last_facts: Option<index::Execution>,
    ) -> Result<Candidate, String> {
        Application {
            progress,
            expected,
            last_facts,
        }
        .run(self, admission)
    }
    fn application_step_source(
        &mut self,
        expected: &Refs,
        admission: &mut Lease,
    ) -> Result<Arc<Refs>, String> {
        let refs = self.check_integration_source(expected, admission)?;
        if self.read_sequence(&refs, admission)?.family.is_some() {
            return Err("Another Git-owned sequence appeared before the next ordered step".into());
        }
        self.check_integration_source(expected, admission)
    }
    pub(super) fn verify_application_step(
        &mut self,
        progress: &ApplyProgress,
        after: &Refs,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let Some(head) = after.head else {
            return Ok(false);
        };
        if head == progress.before
            || after.head_ref != progress.plan.refs.head_ref
            || after
                .observation
                .status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete || !s.paths.is_empty())
        {
            return Ok(false);
        }
        if !self.verify_applied_commit(progress, after, head, admission)? {
            return Ok(false);
        }
        self.verify_applied_tree(after, head, admission)
    }
    fn verify_applied_commit(
        &mut self,
        progress: &ApplyProgress,
        after: &Refs,
        head: ObjectId,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let bytes = self.command(
            &["--no-replace-objects", "cat-file", "commit", &head.hex()],
            Phase::Verifying,
            false,
        )?;
        admission.grow(bytes.len() * 2)?;
        let (parents, message) = commit::commit_object(&bytes, after.observation.identity.format)?;
        let step = &progress.plan.steps[progress.next];
        if parents != [progress.before] {
            return Ok(false);
        }
        if progress.plan.kind == ApplyKind::CherryPick {
            let author = bytes
                .split(|b| *b == b'\n')
                .find_map(|s| s.strip_prefix(b"author "));
            if message != step.message || author != Some(step.author.as_slice()) {
                return Ok(false);
            }
        } else if !message
            .windows(step.object.hex().len())
            .any(|w| w == step.object.hex().as_bytes())
        {
            return Ok(false);
        }
        Ok(true)
    }
    fn verify_applied_tree(
        &mut self,
        after: &Refs,
        head: ObjectId,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let tree = self.command(
            &["ls-tree", "--full-tree", "-r", "-z", &head.hex()],
            Phase::Verifying,
            false,
        )?;
        let index = self.command(&["ls-files", "--stage", "-z"], Phase::Verifying, false)?;
        let count = tree.iter().chain(&index).filter(|b| **b == 0).count();
        admission.grow(
            (tree.len() + index.len()) * 2 + count * 2 * std::mem::size_of::<index::Entry>(),
        )?;
        Ok(index::entries(
            &tree,
            after.observation.identity.format,
            false,
            self.policy.max_status_paths,
        )? == index::entries(
            &index,
            after.observation.identity.format,
            true,
            self.policy.max_status_paths,
        )?)
    }
}

/// An ordered application owns the exact next position, source and last verified child.
struct Application {
    progress: ApplyProgress,
    expected: Arc<Refs>,
    last_facts: Option<index::Execution>,
}
impl Application {
    fn run(mut self, runner: &mut Runner, admission: &mut Lease) -> Result<Candidate, String> {
        while self.progress.next < self.progress.plan.steps.len() {
            let refs = self.source(runner, admission)?;
            let args = self.arguments(&refs)?;
            let (executed, facts) = runner.execute_change(&args, None);
            if runner.retained.is_some() {
                runner.restore_execution(facts, true);
                return Err(
                    "Ordered revision cleanup remains owned; no abort or rollback is inferred"
                        .into(),
                );
            }
            let step = ApplicationStep {
                refs,
                executed,
                facts,
            };
            let result = self.observe(runner, &step, admission);
            match result {
                Ok((candidate, None)) => {
                    runner.restore_execution(step.facts, step.executed.is_err());
                    return Ok(candidate);
                }
                Ok((candidate, Some(next))) => {
                    self = next;
                    self.last_facts = Some(step.facts.clone());
                    runner.restore_execution(step.facts, false);
                    if self.progress.next == self.progress.plan.steps.len() {
                        return Ok(candidate);
                    }
                }
                Err(e) => {
                    runner.restore_execution(step.facts, step.executed.is_err());
                    runner.operation_failure=Some("Ordered revision effects remain unverified; refresh actual Git state; no automatic retry".into());
                    return Err(e);
                }
            }
        }
        Err("Ordered request has no remaining revision".into())
    }
    fn source(&mut self, runner: &mut Runner, admission: &mut Lease) -> Result<Arc<Refs>, String> {
        match runner.application_step_source(&self.expected, admission) {
            Ok(r) => Ok(r),
            Err(e) => {
                if let Some(facts) = self.last_facts.take() {
                    runner.restore_execution(facts, false);
                }
                runner.operation_failure = Some(format!(
                    "Ordered request stopped after {} verified steps: {e}",
                    self.progress.completed.len()
                ));
                Err(e)
            }
        }
    }
    fn arguments(&mut self, refs: &Refs) -> Result<Vec<OsString>, String> {
        let step = &self.progress.plan.steps[self.progress.next];
        self.progress.before = refs.head.ok_or("Ordered application lost HEAD")?;
        let mut args: Vec<OsString> = vec![
            "--no-replace-objects".into(),
            self.progress.plan.kind.label().into(),
            "--no-edit".into(),
        ];
        if let Some(mainline) = step.mainline {
            args.extend(["--mainline".into(), mainline.to_string().into()]);
        }
        args.extend(["--".into(), step.object.hex().into()]);
        Ok(args)
    }
    fn observe(
        mut self,
        runner: &mut Runner,
        step: &ApplicationStep,
        admission: &mut Lease,
    ) -> Result<(Candidate, Option<Self>), String> {
        let after = runner.integration_refs(&self.progress.plan.refs.observation, admission)?;
        let sequence = runner.read_sequence(&after, admission)?;
        if sequence.family.is_some() {
            return self.stopped(runner, sequence, admission);
        }
        if !self.verified(runner, &after, step, admission)? {
            return Ok((Candidate::Sequence(sequence), None));
        }
        let head = after.head.ok_or("Ordered result lost HEAD")?;
        self.progress.completed.push(head);
        self.progress.next += 1;
        self.progress.before = head;
        runner.effects = Effects::Verified;
        self.expected = after;
        Ok((Candidate::Sequence(sequence), Some(self)))
    }
    fn stopped(
        mut self,
        runner: &mut Runner,
        mut sequence: Sequence,
        admission: &mut Lease,
    ) -> Result<(Candidate, Option<Self>), String> {
        let size = self.progress.bytes();
        admission.grow(size)?;
        self.progress._admission = admission.split(size);
        runner.bind_application(&mut sequence, Arc::new(self.progress))?;
        runner.in_progress = true;
        runner.operation_failure=Some("Ordered revisions stopped in progress; explicit checked continuation, skip or abort is required".into());
        Ok((Candidate::Sequence(sequence), None))
    }
    fn verified(
        &self,
        runner: &mut Runner,
        after: &Refs,
        step: &ApplicationStep,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let other_refs = step.other_refs(after);
        if step.executed.is_err()
            || !other_refs
            || !runner.verify_application_step(&self.progress, after, admission)?
        {
            runner.failure_verified = step.executed.is_err()
                && after.same(&step.refs)
                && after.observation.status == step.refs.observation.status;
            runner.operation_failure=Some("Ordered revision failed or actual branch, parents, authored message, index or disk postcondition differs; no automatic retry".into());
            return Ok(false);
        }
        Ok(true)
    }
}
/// One actual changing child retains the freshly checked source through publication.
struct ApplicationStep {
    refs: Arc<Refs>,
    executed: Result<(), String>,
    facts: index::Execution,
}
impl ApplicationStep {
    fn other_refs(&self, after: &Refs) -> bool {
        after.completeness == Completeness::Complete
            && after.entries.len() == self.refs.entries.len()
            && self
                .refs
                .entries
                .iter()
                .filter(|r| r.name != self.refs.head_ref)
                .all(|r| {
                    after
                        .reference(&r.name)
                        .is_some_and(|a| a.object == r.object && a.symbolic == r.symbolic)
                })
    }
}
