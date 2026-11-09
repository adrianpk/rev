use super::*;

impl Runner {
    pub(super) fn finish(
        mut self,
        mut observed: Result<Candidate, String>,
        mut admission: Lease,
    ) -> Finished {
        if self.cancel.is_cancelled() {
            self.end = capture::End::Cancelled;
            observed = Err("Git observation cancelled before publication".into());
        } else if Instant::now() >= self.deadline {
            self.end = capture::End::TimedOut;
            observed = Err("Git observation timed out before publication".into());
        }
        let progress = *self.progress.lock().expect("Git progress owner");
        let outcome = self.finish_outcome(observed.is_ok());
        let diagnostic = self.finish_diagnostic(observed.as_ref().err());
        let mut record = Record {
            id: progress.operation,
            kind: progress.kind,
            context: progress.context,
            started: progress.started,
            ended: Instant::now(),
            outcome,
            effects: self.effects,
            argv: self.argv,
            exit: self.exit,
            stdout: self.stdout,
            stderr: self.stderr,
            output_truncated: self.truncated,
            diagnostic,
            admission: admission.split(0),
        };
        record.admission = admission.split(record.bytes() + 4096);
        let publication = observed.and_then(|candidate| {
            candidate.publish(progress, admission, &self.cancel, self.deadline)
        });
        record_publication(
            &mut record,
            &publication,
            &self.cancel,
            self.deadline,
            self.private_output,
        );
        Finished {
            publication,
            record,
            retained: self.retained.map(|child| Retained {
                child,
                _driver: self.driver_admission,
            }),
        }
    }
    fn finish_outcome(&self, observed: bool) -> Outcome {
        if self.retained.is_some() {
            return Outcome::CleanupPending;
        }
        match self.end {
            capture::End::Cancelled => Outcome::Cancelled,
            capture::End::TimedOut => Outcome::TimedOut,
            capture::End::OutputLimit => Outcome::OutputLimit,
            _ => self.operation_outcome(observed),
        }
    }
    fn operation_outcome(&self, observed: bool) -> Outcome {
        if self.in_progress {
            return Outcome::InProgress;
        }
        if self.operation_failure.is_some()
            && self.effects == Effects::Possible
            && !self.failure_verified
        {
            return Outcome::Indeterminate;
        }
        if self.operation_failure.is_some() {
            return Outcome::Failed;
        }
        if observed {
            self.completion.unwrap_or(Outcome::Verified)
        } else {
            Outcome::Failed
        }
    }
    fn finish_diagnostic(&mut self, error: Option<&String>) -> Option<String> {
        let diagnostic = self.operation_failure.take().or_else(|| error.cloned());
        if !self.private_output {
            return diagnostic;
        }
        let mut text = diagnostic.unwrap_or("Checked Git request completed".into());
        if let Some(reason) = self.transport_diagnostic {
            text.push_str("; ");
            text.push_str(reason);
        }
        text.push_str("; raw transport/configuration output withheld to protect credentials");
        Some(text)
    }
}

impl Candidate {
    fn publish(
        self,
        progress: Progress,
        admission: Lease,
        cancel: &Cancellation,
        deadline: Instant,
    ) -> Result<Publication, String> {
        match self {
            Self::FamilyPlan(plan) => Ok(Publication::FamilyPlan(plan)),
            Self::FamilyResult(result) => Ok(Publication::FamilyResult(result)),
            Self::Worktrees(worktrees) => Ok(Publication::Worktrees(worktrees)),
            Self::WorktreePlan(plan) => Ok(Publication::WorktreePlan(plan)),
            Self::Stashes(stashes) => Ok(Publication::Stashes(stashes)),
            Self::StashDetail(detail) => Ok(Publication::StashDetail(detail)),
            Self::StashPlan(plan) => Ok(Publication::StashPlan(plan)),
            Self::StashResult { stashes, sequence } => {
                Ok(Publication::StashResult { stashes, sequence })
            }
            candidate => candidate.publish_operation(progress, admission, cancel, deadline),
        }
    }
    fn publish_operation(
        self,
        progress: Progress,
        admission: Lease,
        cancel: &Cancellation,
        deadline: Instant,
    ) -> Result<Publication, String> {
        match self {
            Self::BisectPlan(plan) => Ok(Publication::BisectPlan(plan)),
            Self::ApplicationPlan(plan) => Ok(Publication::ApplicationPlan(plan)),
            Self::Autosquash(plan) => Ok(Publication::Autosquash(plan)),
            Self::ConflictExplanation(snapshot) => Ok(Publication::ConflictExplanation(snapshot)),
            Self::Resolution(plan) => Ok(Publication::Resolution(plan)),
            Self::Conflict(conflict) => Ok(Publication::Conflict(conflict)),
            Self::Sequence(sequence) => Ok(Publication::Sequence(sequence)),
            candidate => candidate.publish_observation(progress, admission, cancel, deadline),
        }
    }
    fn publish_observation(
        self,
        progress: Progress,
        admission: Lease,
        cancel: &Cancellation,
        deadline: Instant,
    ) -> Result<Publication, String> {
        match self {
            Self::PushIntent(intent) => Ok(Publication::PushIntent(intent)),
            Self::TagIntent(intent) => Ok(Publication::TagIntent(intent)),
            Self::Refs { refs, changed } => Ok(Publication::Refs { refs, changed }),
            Self::History(result) => Ok(Publication::History(result)),
            Self::Status(identity, status) => Ok(Publication::Status(Observation {
                context: progress.context.unwrap_or(ContextId(progress.operation.0)),
                generation: progress.operation.0,
                identity,
                status,
                admission,
            })),
            Self::Diff(prepared) => diff::prepare(
                progress.operation,
                prepared,
                admission,
                &cancel.bounded(deadline),
            )
            .map(Publication::Diff),
            Self::Composition(prepared) => {
                let mut composition = Composition {
                    observation: prepared.observation,
                    mode: prepared.mode,
                    message: prepared.message,
                    parents: prepared.parents,
                    fixup_prefix: prepared.fixup_prefix,
                    admission,
                };
                composition.admission.retain(composition.bytes());
                Ok(Publication::Composition(composition))
            }
            _ => unreachable!("candidate publication dispatched by its operation family"),
        }
    }
}

fn record_publication(
    record: &mut Record,
    publication: &Result<Publication, String>,
    cancel: &Cancellation,
    deadline: Instant,
    private_output: bool,
) {
    if let Err(error) = publication {
        if record.outcome == Outcome::Verified {
            record.outcome = failed_publication_outcome(cancel, deadline);
        }
        if !private_output {
            record.diagnostic = Some(error.clone());
        }
    }
    if let Ok(Publication::Diff(detail)) = publication
        && let DiffKind::Malformed(reason) = &detail.kind
    {
        record.outcome = Outcome::Failed;
        record.diagnostic = Some(reason.clone());
    }
}
fn failed_publication_outcome(cancel: &Cancellation, deadline: Instant) -> Outcome {
    if cancel.is_cancelled() {
        return Outcome::Cancelled;
    }
    if Instant::now() >= deadline {
        Outcome::TimedOut
    } else {
        Outcome::Failed
    }
}
