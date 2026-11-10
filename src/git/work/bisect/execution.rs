use super::*;

/// One native change retains its exact original lineage through verification.
pub(super) struct Change<'a> {
    pub plan: &'a Arc<BisectPlan>,
    pub control: Option<(&'a Arc<Sequence>, BisectAction)>,
}

/// A changing child's capture result and facts must be verified together.
struct Attempt {
    result: Result<(), String>,
    facts: index::Execution,
}

impl Change<'_> {
    pub(super) fn run(
        &self,
        runner: &mut Runner,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let args = self.arguments(runner, admission)?;
        let (result, facts) = runner.execute_change(&args, None);
        let attempt = Attempt { result, facts };
        if runner.retained.is_some() {
            runner.restore_execution(attempt.facts, true);
            return Err(
                "Manual bisect cleanup remains owned; cancellation does not reset it".into(),
            );
        }
        let result = self.observe(runner, &attempt, admission);
        runner.restore_execution(attempt.facts, attempt.result.is_err());
        result
    }
    fn arguments(
        &self,
        runner: &mut Runner,
        admission: &mut Lease,
    ) -> Result<Vec<OsString>, String> {
        if let Some((sequence, action)) = self.control {
            runner.bisect_control_arguments(sequence, action, admission)
        } else {
            runner.bisect_start_arguments(self.plan, admission)
        }
    }
    fn observe(
        &self,
        runner: &mut Runner,
        attempt: &Attempt,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let refs = runner.integration_refs(&self.plan.refs.observation, admission)?;
        let mut after = runner.read_sequence(&refs, admission)?;
        let reset = self.control.is_some_and(|(_, a)| a == BisectAction::Reset);
        let verified = if reset {
            self.verify_reset(runner, &after, attempt, admission)?
        } else {
            self.verify_classification(runner, &mut after, attempt, admission)?
        };
        if !verified {
            runner.operation_failure = Some("Manual bisect failed or exact classification/original-context postcondition differs".into());
        }
        Ok(Candidate::Sequence(after))
    }
    fn verify_reset(
        &self,
        runner: &mut Runner,
        after: &Sequence,
        attempt: &Attempt,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let dangling = runner.command(
            &["for-each-ref", "--format=%(refname)", "refs/bisect/"],
            Phase::Verifying,
            false,
        )?;
        if attempt.result.is_ok()
            && after.family.is_none()
            && !after.files.iter().any(|f| f.name.starts_with(b"BISECT_"))
            && dangling.is_empty()
        {
            runner.check_integration_source(&self.plan.refs, admission)?;
            runner.effects = Effects::Verified;
            return Ok(true);
        }
        Ok(false)
    }
    fn verify_classification(
        &self,
        runner: &mut Runner,
        after: &mut Sequence,
        attempt: &Attempt,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        runner.bind_bisect(after, Arc::clone(self.plan))?;
        let state = after
            .bisect
            .as_ref()
            .ok_or("Bisect candidate state is missing")?;
        let clean = after
            .refs
            .observation
            .status
            .as_ref()
            .is_some_and(|s| s.completeness == Completeness::Complete && s.paths.is_empty());
        let classified = self.classified(state, admission)?;
        // Exit 2 alone cannot prove ambiguity: the observed graph must agree.
        let ambiguous_exit =
            state.result == BisectResult::Ambiguous && runner.bisect_exit_two(&attempt.facts);
        if clean && classified && (attempt.result.is_ok() || ambiguous_exit) {
            runner.effects = Effects::Verified;
            runner.in_progress = state.result != BisectResult::Found(state.bad);
            return Ok(true);
        }
        Ok(false)
    }
    fn classified(&self, state: &BisectState, admission: &mut Lease) -> Result<bool, String> {
        if let Some((before, action)) = self.control {
            self.classified_control(before, action, state, admission)
        } else {
            Ok(state.bad == self.plan.bad
                && state.good == [self.plan.good]
                && state.skipped.is_empty())
        }
    }
    fn classified_control(
        &self,
        before: &Sequence,
        action: BisectAction,
        state: &BisectState,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let old = before
            .bisect
            .as_ref()
            .ok_or("Bisect review lacks boundaries")?;
        let candidate = before.refs.head.ok_or("Bisect review lacks candidate")?;
        admission
            .grow((old.good.len() + old.skipped.len() + 2) * std::mem::size_of::<ObjectId>())?;
        let mut good = Vec::with_capacity(old.good.len() + 1);
        let mut skipped = Vec::with_capacity(old.skipped.len() + 1);
        good.extend_from_slice(&old.good);
        skipped.extend_from_slice(&old.skipped);
        let bad = if action == BisectAction::Bad {
            candidate
        } else {
            old.bad
        };
        if action == BisectAction::Good {
            good.push(candidate);
        }
        if action == BisectAction::Skip {
            skipped.push(candidate);
        }
        good.sort_by_key(|id| id.bytes);
        skipped.sort_by_key(|id| id.bytes);
        Ok(state.bad == bad && state.good == good && state.skipped == skipped)
    }
}

impl Runner {
    fn bisect_control_arguments(
        &mut self,
        sequence: &Arc<Sequence>,
        action: BisectAction,
        admission: &mut Lease,
    ) -> Result<Vec<OsString>, String> {
        let actual = self.checked_sequence(sequence, admission)?;
        actual.bisect_admission(action)?;
        self.bisect_classification_capacity(&actual, action)?;
        let mut args = vec![
            "--no-replace-objects".into(),
            "bisect".into(),
            action.label().into(),
        ];
        if action != BisectAction::Reset {
            args.push(
                actual
                    .refs
                    .head
                    .ok_or("Bisect candidate lost HEAD")?
                    .hex()
                    .into(),
            );
        }
        self.checked_sequence(sequence, admission)?;
        Ok(args)
    }
    fn bisect_classification_capacity(
        &self,
        actual: &Sequence,
        action: BisectAction,
    ) -> Result<(), String> {
        if matches!(action, BisectAction::Good | BisectAction::Skip)
            && actual.bisect.as_ref().is_some_and(|b| {
                b.good.len() + b.skipped.len() + 1 >= self.policy.max_refs.min(256)
            })
        {
            return Err("Bisect classification exceeds configured complete-ref admission".into());
        }
        Ok(())
    }
    fn bisect_start_arguments(
        &mut self,
        plan: &BisectPlan,
        admission: &mut Lease,
    ) -> Result<Vec<OsString>, String> {
        let current = self.check_integration_source(&plan.refs, admission)?;
        if self.read_sequence(&current, admission)?.family.is_some() {
            return Err("Another Git sequence appeared before bisect start".into());
        }
        self.check_integration_source(&plan.refs, admission)?;
        Ok(vec![
            "--no-replace-objects".into(),
            "bisect".into(),
            "start".into(),
            plan.bad.hex().into(),
            plan.good.hex().into(),
            "--".into(),
        ])
    }
}
