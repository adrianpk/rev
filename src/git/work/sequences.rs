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
        mut progress: ApplyProgress,
        mut expected: Arc<Refs>,
        admission: &mut Lease,
        mut last_facts: Option<index::Execution>,
    ) -> Result<Candidate, String> {
        while progress.next < progress.plan.steps.len() {
            let checked = (|| {
                let refs = self.check_integration_source(&expected, admission)?;
                if self.read_sequence(&refs, admission)?.family.is_some() {
                    return Err(
                        "Another Git-owned sequence appeared before the next ordered step".into(),
                    );
                }
                self.check_integration_source(&expected, admission)
            })();
            let refs = match checked {
                Ok(r) => r,
                Err(e) => {
                    if let Some(facts) = last_facts {
                        self.restore_execution(facts, false);
                    }
                    self.operation_failure = Some(format!(
                        "Ordered request stopped after {} verified steps: {e}",
                        progress.completed.len()
                    ));
                    return Err(e);
                }
            };
            let step = &progress.plan.steps[progress.next];
            progress.before = refs.head.ok_or("Ordered application lost HEAD")?;
            let mut args: Vec<OsString> = vec![
                "--no-replace-objects".into(),
                progress.plan.kind.label().into(),
                "--no-edit".into(),
            ];
            if let Some(mainline) = step.mainline {
                args.extend(["--mainline".into(), mainline.to_string().into()]);
            }
            args.extend(["--".into(), step.object.hex().into()]);
            let (executed, facts) = self.execute_change(&args, None);
            if self.retained.is_some() {
                self.restore_execution(facts, true);
                return Err(
                    "Ordered revision cleanup remains owned; no abort or rollback is inferred"
                        .into(),
                );
            }
            let result = (|| {
                let after = self.integration_refs(&progress.plan.refs.observation, admission)?;
                let mut sequence = self.read_sequence(&after, admission)?;
                if sequence.family.is_some() {
                    let size = progress.bytes();
                    admission.grow(size)?;
                    progress._admission = admission.split(size);
                    self.bind_application(&mut sequence, Arc::new(progress))?;
                    self.in_progress = true;
                    self.operation_failure=Some("Ordered revisions stopped in progress; explicit checked continuation, skip or abort is required".into());
                    return Ok((Candidate::Sequence(sequence), None));
                }
                let other_refs = after.completeness == Completeness::Complete
                    && after.entries.len() == refs.entries.len()
                    && refs
                        .entries
                        .iter()
                        .filter(|r| r.name != refs.head_ref)
                        .all(|r| {
                            after
                                .reference(&r.name)
                                .is_some_and(|a| a.object == r.object && a.symbolic == r.symbolic)
                        });
                if executed.is_err()
                    || !other_refs
                    || !self.verify_application_step(&progress, &after, admission)?
                {
                    self.failure_verified = executed.is_err()
                        && after.same(&refs)
                        && after.observation.status == refs.observation.status;
                    self.operation_failure=Some("Ordered revision failed or actual branch, parents, authored message, index or disk postcondition differs; no automatic retry".into());
                    return Ok((Candidate::Sequence(sequence), None));
                }
                let head = after.head.ok_or("Ordered result lost HEAD")?;
                progress.completed.push(head);
                progress.next += 1;
                progress.before = head;
                self.effects = Effects::Verified;
                Ok((Candidate::Sequence(sequence), Some((progress, after))))
            })();
            match result {
                Ok((candidate, None)) => {
                    self.restore_execution(facts, executed.is_err());
                    return Ok(candidate);
                }
                Ok((candidate, Some((next, after)))) => {
                    progress = next;
                    expected = after;
                    last_facts = Some(facts.clone());
                    self.restore_execution(facts, false);
                    if progress.next == progress.plan.steps.len() {
                        return Ok(candidate);
                    }
                }
                Err(e) => {
                    self.restore_execution(facts, executed.is_err());
                    self.operation_failure=Some("Ordered revision effects remain unverified; refresh actual Git state; no automatic retry".into());
                    return Err(e);
                }
            }
        }
        Err("Ordered request has no remaining revision".into())
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
