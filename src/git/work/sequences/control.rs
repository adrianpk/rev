use super::*;

impl Runner {
    pub(in crate::git::work) fn control_application(
        &mut self,
        expected: Arc<Sequence>,
        action: SequenceControl,
        family: Option<SequenceFamily>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let current = self.checked_sequence(&expected, admission)?;
        current.control_admission(action, family)?;
        let retained = current
            .application
            .as_ref()
            .ok_or("Ordered lineage is unavailable")?;
        admission.grow(
            std::mem::size_of::<ApplyProgress>()
                + retained.plan.steps.len() * std::mem::size_of::<ObjectId>(),
        )?;
        let mut progress = ApplyProgress {
            plan: retained.plan.clone(),
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
        let args = [
            "--no-replace-objects".into(),
            progress.plan.kind.label().into(),
            format!("--{}", action.label()).into(),
        ];
        self.checked_sequence(&expected, admission)?;
        self.accept_message = action == SequenceControl::Continue;
        let (executed, facts) = self.execute_change(&args, None);
        self.accept_message = false;
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Ordered control cleanup remains owned; cancellation does not abort".into(),
            );
        }
        let result = (|| {
            let refs = self.integration_refs(&expected.refs.observation, admission)?;
            let mut after = self.read_sequence(refs.clone(), admission)?;
            if after.family.is_some() {
                if self.bind_application(&mut after, retained.clone()).is_ok() {
                    self.in_progress = true;
                }
                self.operation_failure = Some("Ordered step remains in progress; inspect exact Git state before another explicit control".into());
                return Ok(Candidate::Sequence(after));
            }
            let clean =
                refs.observation.status.as_ref().is_some_and(|s| {
                    s.completeness == Completeness::Complete && s.paths.is_empty()
                });
            let same_refs = refs.completeness == Completeness::Complete
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
                    });
            let valid = if action == SequenceControl::Continue {
                self.verify_application_step(&progress, &refs, admission)?
                    && self.command(&["ls-files", "--stage", "-z"], Phase::Verifying, false)?
                        == expected.index
            } else {
                refs.head == Some(progress.before) && clean
            };
            if executed.is_err() || !same_refs || !valid {
                self.failure_verified = executed.is_err() && expected.same(&after);
                self.operation_failure = Some("Ordered control failed or exact parent/message/index/branch postcondition differs; no automatic retry".into());
                return Ok(Candidate::Sequence(after));
            }
            self.effects = Effects::Verified;
            Ok(Candidate::Sequence(after))
        })();
        let after = match result {
            Ok(Candidate::Sequence(after)) if self.effects == Effects::Verified => after,
            other => {
                self.restore_execution(facts, executed.is_err());
                return other;
            }
        };
        if action == SequenceControl::Abort {
            // Git owns the stopped single step. The explicitly confirmed
            // ordered abort additionally restores its verified completed prefix.
            let original = progress
                .plan
                .refs
                .head
                .ok_or("Original ordered HEAD is unavailable")?;
            if progress.before != original {
                let preflight = self
                    .check_integration_source(&after.refs, admission)
                    .and_then(|refs| {
                        if self.read_sequence(refs, admission)?.family.is_some() {
                            Err(
                                "Another sequence appeared before restoring the ordered original"
                                    .into(),
                            )
                        } else {
                            Ok(())
                        }
                    });
                if let Err(e) = preflight {
                    self.restore_execution(facts, false);
                    self.effects = Effects::Possible;
                    return Err(e);
                }
                let args = [
                    "--no-replace-objects".into(),
                    "reset".into(),
                    "--merge".into(),
                    original.hex().into(),
                ];
                let (reset, reset_facts) = self.execute_change(&args, None);
                let checked = (|| {
                    if self.retained.is_some() {
                        return Err("Ordered original restoration cleanup remains owned".into());
                    }
                    let refs = self.check_integration_source(&progress.plan.refs, admission)?;
                    let after = self.read_sequence(refs, admission)?;
                    if reset.is_err() || after.family.is_some() {
                        return Err(
                            "Ordered original restoration failed or a sequence remains".into()
                        );
                    }
                    self.effects = Effects::Verified;
                    Ok(Candidate::Sequence(after))
                })();
                self.restore_execution(reset_facts, reset.is_err());
                if checked.is_ok() {
                    self.completion = Some(Outcome::Aborted);
                }
                return checked;
            }
            self.completion = Some(Outcome::Aborted);
            self.restore_execution(facts, false);
            return Ok(Candidate::Sequence(after));
        }
        if action == SequenceControl::Continue {
            progress
                .completed
                .push(after.refs.head.ok_or("Continued ordered step lost HEAD")?);
        }
        progress.next += 1;
        progress.before = after.refs.head.ok_or("Ordered control lost HEAD")?;
        if progress.next == progress.plan.steps.len() {
            self.completion = Some(if action == SequenceControl::Skip {
                Outcome::Skipped
            } else {
                Outcome::Verified
            });
            self.restore_execution(facts, false);
            return Ok(Candidate::Sequence(after));
        }
        let result = self.apply_steps(progress, after.refs, admission, Some(facts));
        if result.is_ok() && self.operation_failure.is_none() && action == SequenceControl::Skip {
            self.completion = Some(Outcome::Skipped);
        }
        result
    }
}
