use super::*;
use crate::git::bisect::Request;
mod observation;

impl Runner {
    pub(super) fn bisect(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let result = match request {
            Request::Prepare {
                refs, bad, good, ..
            } => self
                .prepare_bisect(refs, &bad, &good, admission)
                .map(Candidate::BisectPlan),
            Request::Start(plan) => self.change_bisect(&plan, &None, admission),
            Request::Control { sequence, action } => {
                let plan = sequence
                    .bisect
                    .as_ref()
                    .and_then(|b| b.plan.clone())
                    .ok_or("Bisect original lineage is unavailable")?;
                self.change_bisect(&plan, &Some((sequence, action)), admission)
            }
        };
        if result.is_err() && self.effects == Effects::Possible {
            self.operation_failure.get_or_insert_with(||"Manual bisect effects remain unverified; refresh actual Git state; no automatic retry or inferred reset".into());
        }
        result
    }
    fn prepare_bisect(
        &mut self,
        refs: Arc<Refs>,
        bad: &str,
        good: &str,
        admission: &mut Lease,
    ) -> Result<BisectPlan, String> {
        if self.policy.max_refs < 2 {
            return Err("Bisect boundaries require admission for at least two refs".into());
        }
        let current = self.check_integration_source(&refs, admission)?;
        if current.completeness != Completeness::Complete
            || current.head.is_none()
            || !current.head_ref.starts_with(b"refs/heads/")
            || self.read_sequence(&current, admission)?.family.is_some()
        {
            return Err(
                "Manual bisect starts from a checked existing branch without another sequence"
                    .into(),
            );
        }
        let mut resolve = |text: &str| -> Result<ObjectId, String> {
            let bytes = self.command(
                &[
                    "--no-replace-objects",
                    "rev-parse",
                    "--verify",
                    "--end-of-options",
                    &format!("{text}^{{commit}}"),
                ],
                Phase::Checking,
                false,
            )?;
            ObjectId::parse(
                bytes.strip_suffix(b"\n").unwrap_or(&bytes),
                refs.observation.identity.format,
            )
        };
        let bad = resolve(bad)?;
        let good = resolve(good)?;
        if bad == good || !self.bisect_ancestor(good, bad)? {
            return Err(
                "Bisect requires distinct full bad/good commits with good ancestral to bad".into(),
            );
        }
        self.check_integration_source(&refs, admission)?;
        admission.grow(std::mem::size_of::<BisectPlan>())?;
        Ok(BisectPlan {
            refs,
            bad,
            good,
            _admission: admission.split(std::mem::size_of::<BisectPlan>()),
        })
    }
    pub(in crate::git::work) fn bind_bisect(
        &mut self,
        sequence: &mut Sequence,
        plan: Arc<BisectPlan>,
    ) -> Result<(), String> {
        let start = sequence
            .files
            .iter()
            .find(|f| f.name == b"BISECT_START")
            .map(|f| f.bytes.strip_suffix(b"\n").unwrap_or(&f.bytes));
        let state = sequence
            .bisect
            .as_mut()
            .ok_or("Actual bisect has no compatible complete candidate state")?;
        if sequence.family != Some(SequenceFamily::Bisect)
            || start != plan.refs.head_ref.strip_prefix(b"refs/heads/")
            || sequence.refs.entries != plan.refs.entries
            || sequence.refs.remotes != plan.refs.remotes
            || sequence.refs.completeness != Completeness::Complete
            || !state.good.contains(&plan.good)
            || !self.bisect_ancestor(state.bad, plan.bad)?
        {
            return Err(
                "Actual bisect boundaries/original branch differ from retained exact lineage"
                    .into(),
            );
        }
        sequence._admission.grow(plan.refs.head_ref.len())?;
        sequence.original = plan.refs.head;
        sequence.branch = plan.refs.head_ref.clone();
        state.plan = Some(plan);
        Ok(())
    }
    fn change_bisect(
        &mut self,
        plan: &Arc<BisectPlan>,
        control: &Option<(Arc<Sequence>, BisectAction)>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let args: Vec<OsString> = if let Some((sequence, action)) = &control {
            let actual = self.checked_sequence(sequence, admission)?;
            actual.bisect_admission(*action)?;
            if matches!(action, BisectAction::Good | BisectAction::Skip)
                && actual.bisect.as_ref().is_some_and(|b| {
                    b.good.len() + b.skipped.len() + 1 >= self.policy.max_refs.min(256)
                })
            {
                return Err(
                    "Bisect classification exceeds configured complete-ref admission".into(),
                );
            }
            let mut args = vec![
                "--no-replace-objects".into(),
                "bisect".into(),
                action.label().into(),
            ];
            if *action != BisectAction::Reset {
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
            args
        } else {
            let current = self.check_integration_source(&plan.refs, admission)?;
            if self.read_sequence(&current, admission)?.family.is_some() {
                return Err("Another Git sequence appeared before bisect start".into());
            }
            self.check_integration_source(&plan.refs, admission)?;
            vec![
                "--no-replace-objects".into(),
                "bisect".into(),
                "start".into(),
                plan.bad.hex().into(),
                plan.good.hex().into(),
                "--".into(),
            ]
        };
        let (executed, facts) = self.execute_change(&args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Manual bisect cleanup remains owned; cancellation does not reset it".into(),
            );
        }
        let result = (|| {
            let refs = self.integration_refs(&plan.refs.observation, admission)?;
            let mut after = self.read_sequence(&refs, admission)?;
            let reset = control
                .as_ref()
                .is_some_and(|(_, a)| *a == BisectAction::Reset);
            if reset {
                let dangling = self.command(
                    &["for-each-ref", "--format=%(refname)", "refs/bisect/"],
                    Phase::Verifying,
                    false,
                )?;
                if executed.is_ok()
                    && after.family.is_none()
                    && !after.files.iter().any(|f| f.name.starts_with(b"BISECT_"))
                    && dangling.is_empty()
                {
                    self.check_integration_source(&plan.refs, admission)?;
                    self.effects = Effects::Verified;
                    return Ok(Candidate::Sequence(after));
                }
            } else {
                self.bind_bisect(&mut after, std::sync::Arc::clone(plan))?;
                let state = after
                    .bisect
                    .as_ref()
                    .ok_or("Bisect candidate state is missing")?;
                let clean = after.refs.observation.status.as_ref().is_some_and(|s| {
                    s.completeness == Completeness::Complete && s.paths.is_empty()
                });
                let classified = if let Some((before, action)) = &control {
                    let old = before
                        .bisect
                        .as_ref()
                        .ok_or("Bisect review lacks boundaries")?;
                    let candidate = before.refs.head.ok_or("Bisect review lacks candidate")?;
                    admission.grow(
                        (old.good.len() + old.skipped.len() + 2) * std::mem::size_of::<ObjectId>(),
                    )?;
                    let mut good = Vec::with_capacity(old.good.len() + 1);
                    let mut skipped = Vec::with_capacity(old.skipped.len() + 1);
                    good.extend_from_slice(&old.good);
                    skipped.extend_from_slice(&old.skipped);
                    let bad = if *action == BisectAction::Bad {
                        candidate
                    } else {
                        old.bad
                    };
                    if *action == BisectAction::Good {
                        good.push(candidate);
                    }
                    if *action == BisectAction::Skip {
                        skipped.push(candidate);
                    }
                    good.sort_by_key(|id| id.bytes);
                    skipped.sort_by_key(|id| id.bytes);
                    state.bad == bad && state.good == good && state.skipped == skipped
                } else {
                    state.bad == plan.bad && state.good == [plan.good] && state.skipped.is_empty()
                };
                // Git returns 2 when only skipped candidates remain. Actual
                // bounded graph membership proves ambiguity independently of text.
                let ambiguous_exit =
                    state.result == BisectResult::Ambiguous && self.bisect_exit_two(&facts);
                if clean && classified && (executed.is_ok() || ambiguous_exit) {
                    self.effects = Effects::Verified;
                    self.in_progress = state.result != BisectResult::Found(state.bad);
                    return Ok(Candidate::Sequence(after));
                }
            }
            self.operation_failure=Some("Manual bisect failed or exact classification/original-context postcondition differs".into());
            Ok(Candidate::Sequence(after))
        })();
        self.restore_execution(facts, executed.is_err());
        result
    }
}
