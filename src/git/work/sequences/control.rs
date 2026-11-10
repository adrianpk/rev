use super::*;

impl Runner {
    pub(in crate::git::work) fn control_application(
        &mut self,
        expected: &Arc<Sequence>,
        action: SequenceControl,
        family: Option<SequenceFamily>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let current = self.checked_sequence(expected, admission)?;
        current.control_admission(action, family)?;
        let retained = current
            .application
            .as_ref()
            .ok_or("Ordered lineage is unavailable")?;
        let progress = copy_progress(retained, admission)?;
        Control {
            expected,
            action,
            retained,
            progress,
        }
        .run(self, admission)
    }
}

fn copy_progress(retained: &ApplyProgress, admission: &mut Lease) -> Result<ApplyProgress, String> {
    admission.grow(
        std::mem::size_of::<ApplyProgress>()
            + retained.plan.steps.len() * std::mem::size_of::<ObjectId>(),
    )?;
    let mut progress = ApplyProgress {
        plan: std::sync::Arc::clone(&retained.plan),
        next: retained.next,
        before: retained.before,
        completed: Vec::with_capacity(retained.plan.steps.len()),
        _admission: admission.split(0),
    };
    // This copy remains finite and admitted while the immutable review is
    // retained by the session. A later stop publishes new exact lineage.
    progress.completed.extend_from_slice(&retained.completed);
    let size = progress.bytes();
    progress._admission = admission.split(size);
    Ok(progress)
}

/// Exact stopped request and a finite admitted continuation remain one control owner.
struct Control<'a> {
    expected: &'a Arc<Sequence>,
    action: SequenceControl,
    retained: &'a Arc<ApplyProgress>,
    progress: ApplyProgress,
}
impl Control<'_> {
    fn run(self, runner: &mut Runner, admission: &mut Lease) -> Result<Candidate, String> {
        let expected = self.expected;
        let action = self.action;
        let args = [
            "--no-replace-objects".into(),
            self.progress.plan.kind.label().into(),
            format!("--{}", action.label()).into(),
        ];
        runner.checked_sequence(expected, admission)?;
        runner.accept_message = action == SequenceControl::Continue;
        let (executed, facts) = runner.execute_change(&args, None);
        runner.accept_message = false;
        if runner.retained.is_some() {
            runner.restore_execution(facts, true);
            return Err(
                "Ordered control cleanup remains owned; cancellation does not abort".into(),
            );
        }
        let result = self.observe(runner, executed.is_ok(), admission);
        let after = match result {
            Ok(Candidate::Sequence(after)) if runner.effects == Effects::Verified => after,
            other => {
                runner.restore_execution(facts, executed.is_err());
                return other;
            }
        };
        if action == SequenceControl::Abort {
            return self.abort(runner, after, facts, admission);
        }
        self.resume(runner, after, facts, admission)
    }
    fn observe(
        &self,
        runner: &mut Runner,
        executed: bool,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let expected = self.expected;
        let refs = runner.integration_refs(&expected.refs.observation, admission)?;
        let mut after = runner.read_sequence(&refs, admission)?;
        if after.family.is_some() {
            self.active(runner, &mut after);
            return Ok(Candidate::Sequence(after));
        }
        let clean = refs
            .observation
            .status
            .as_ref()
            .is_some_and(|s| s.completeness == Completeness::Complete && s.paths.is_empty());
        let same_refs = self.same_refs(&refs);
        let valid = self.valid(runner, &refs, clean, admission)?;
        Ok(self.classify(runner, after, executed, same_refs && valid))
    }
    fn active(&self, runner: &mut Runner, after: &mut Sequence) {
        let retained = self.retained;
        if runner
            .bind_application(after, std::sync::Arc::clone(retained))
            .is_ok()
        {
            runner.in_progress = true;
        }
        runner.operation_failure = Some("Ordered step remains in progress; inspect exact Git state before another explicit control".into());
    }
    fn same_refs(&self, refs: &Refs) -> bool {
        let expected = self.expected;
        refs.completeness == Completeness::Complete
            && refs.head_ref == expected.refs.head_ref
            && refs.entries.len() == expected.refs.entries.len()
            && expected
                .refs
                .entries
                .iter()
                .filter(|r| r.name != expected.refs.head_ref)
                .all(|r| {
                    refs.reference(&r.name)
                        .is_some_and(|a| a.object == r.object && a.symbolic == r.symbolic)
                })
    }
    fn valid(
        &self,
        runner: &mut Runner,
        refs: &Refs,
        clean: bool,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let expected = self.expected;
        let action = self.action;
        let progress = &self.progress;
        Ok(if action == SequenceControl::Continue {
            runner.verify_application_step(progress, refs, admission)?
                && runner.command(&["ls-files", "--stage", "-z"], Phase::Verifying, false)?
                    == expected.index
        } else {
            refs.head == Some(progress.before) && clean
        })
    }
    fn classify(
        &self,
        runner: &mut Runner,
        after: Sequence,
        executed: bool,
        valid: bool,
    ) -> Candidate {
        let expected = self.expected;
        if !executed || !valid {
            runner.failure_verified = !executed && expected.same(&after);
            runner.operation_failure = Some("Ordered control failed or exact parent/message/index/branch postcondition differs; no automatic retry".into());
            return Candidate::Sequence(after);
        }
        runner.effects = Effects::Verified;
        Candidate::Sequence(after)
    }
    fn abort(
        &self,
        runner: &mut Runner,
        after: Sequence,
        facts: index::Execution,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let progress = &self.progress;
        // Git owns the stopped single step. The explicitly confirmed
        // ordered abort additionally restores its verified completed prefix.
        let original = progress
            .plan
            .refs
            .head
            .ok_or("Original ordered HEAD is unavailable")?;
        if progress.before != original {
            let preflight = runner.check_ordered_restoration(&after, admission);
            if let Err(e) = preflight {
                runner.restore_execution(facts, false);
                runner.effects = Effects::Possible;
                return Err(e);
            }
            let args = [
                "--no-replace-objects".into(),
                "reset".into(),
                "--merge".into(),
                original.hex().into(),
            ];
            let (reset, reset_facts) = runner.execute_change(&args, None);
            let checked = self.verify_original(runner, reset.is_ok(), admission);
            runner.restore_execution(reset_facts, reset.is_err());
            if checked.is_ok() {
                runner.completion = Some(Outcome::Aborted);
            }
            return checked;
        }
        runner.completion = Some(Outcome::Aborted);
        runner.restore_execution(facts, false);
        Ok(Candidate::Sequence(after))
    }
    fn verify_original(
        &self,
        runner: &mut Runner,
        reset: bool,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let progress = &self.progress;
        if runner.retained.is_some() {
            return Err("Ordered original restoration cleanup remains owned".into());
        }
        let refs = runner.check_integration_source(&progress.plan.refs, admission)?;
        let after = runner.read_sequence(&refs, admission)?;
        if !reset || after.family.is_some() {
            return Err("Ordered original restoration failed or a sequence remains".into());
        }
        runner.effects = Effects::Verified;
        Ok(Candidate::Sequence(after))
    }
    fn resume(
        self,
        runner: &mut Runner,
        after: Sequence,
        facts: index::Execution,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let action = self.action;
        let mut progress = self.progress;
        if action == SequenceControl::Continue {
            progress
                .completed
                .push(after.refs.head.ok_or("Continued ordered step lost HEAD")?);
        }
        progress.next += 1;
        progress.before = after.refs.head.ok_or("Ordered control lost HEAD")?;
        if progress.next == progress.plan.steps.len() {
            runner.completion = Some(if action == SequenceControl::Skip {
                Outcome::Skipped
            } else {
                Outcome::Verified
            });
            runner.restore_execution(facts, false);
            return Ok(Candidate::Sequence(after));
        }
        let result = runner.apply_steps(progress, after.refs, admission, Some(facts));
        if result.is_ok() && runner.operation_failure.is_none() && action == SequenceControl::Skip {
            runner.completion = Some(Outcome::Skipped);
        }
        result
    }
}
impl Runner {
    fn check_ordered_restoration(
        &mut self,
        after: &Sequence,
        admission: &mut Lease,
    ) -> Result<(), String> {
        let refs = self.check_integration_source(&after.refs, admission)?;
        if self.read_sequence(&refs, admission)?.family.is_some() {
            Err("Another sequence appeared before restoring the ordered original".into())
        } else {
            Ok(())
        }
    }
}
