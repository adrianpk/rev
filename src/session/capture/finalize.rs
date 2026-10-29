use super::*;
use crate::{
    buffer::Stamp,
    file::{Outcome, Publication},
    org::capture::{Finalize, Insertion},
};
mod publish;
pub(super) struct Validation {
    pub draft: BufferId,
    pub stamp: Stamp,
    pub target: Box<Resolved>,
    pub work: Option<Finalize>,
}
pub(super) struct Stored {
    pub draft: BufferId,
    pub draft_stamp: Stamp,
    pub source: Stamp,
    pub path: PathBuf,
    pub target: Box<Resolved>,
}
struct Applied {
    source: Stamp,
    outcome: Outcome,
}
impl Session {
    pub(super) fn capture_validating(&self) -> bool {
        self.capture
            .as_ref()
            .is_some_and(|s| matches!(s.stage, Stage::Finalizing(_)))
    }
    pub(super) fn begin_capture_finalize(&mut self) -> Action {
        if self.input.focus() != crate::input::Focus::Editing || self.pending.is_some() {
            return self.message("Finish the current draft input before finalizing");
        }
        if !self.capture_selected() {
            return self.message("Select the capture draft before finalizing");
        }
        let state = self.capture.take().expect("selected capture owner");
        let State { origin, stage } = state;
        match stage {
            Stage::Draft(id, target) => self.validate_capture(origin, id, target),
            Stage::Published(stored) => self.retry_capture_save(origin, *stored),
            stage => {
                self.capture = Some(State { origin, stage });
                self.message("Capture validation is pending; C-g keeps the draft")
            }
        }
    }
    fn validate_capture(&mut self, origin: Origin, id: BufferId, target: Box<Resolved>) -> Action {
        let result = self
            .buffers
            .get(id)
            .map_err(|e| e.to_string())
            .and_then(|draft| {
                target
                    .finalize(
                        &self.buffers,
                        draft,
                        self.buffers.snapshot().settings(),
                        origin.context.now.date,
                    )
                    .map(|work| (work, draft.stamp()))
            });
        match result {
            Ok((work, stamp)) => {
                self.org.cancel();
                self.capture = Some(State {
                    origin,
                    stage: Stage::Finalizing(Box::new(Validation {
                        draft: id,
                        stamp,
                        target,
                        work: Some(work),
                    })),
                });
                self.advance_capture();
                self.message("Validating capture destination; C-g keeps the draft")
            }
            Err(e) => {
                self.capture = Some(State {
                    origin,
                    stage: Stage::Draft(id, target),
                });
                self.error(&e)
            }
        }
    }
    pub(super) fn launch_capture_validation(&mut self, state: State) {
        let State { origin, stage } = state;
        let Stage::Finalizing(mut validation) = stage else {
            unreachable!("owned finalization stage")
        };
        let Some(work) = validation.work.take() else {
            self.capture = Some(State {
                origin,
                stage: Stage::Finalizing(validation),
            });
            return;
        };
        match self.org.start_capture_finalize(validation.stamp, work) {
            Ok(()) => {
                self.capture = Some(State {
                    origin,
                    stage: Stage::Finalizing(validation),
                })
            }
            Err(e) => {
                self.capture = Some(State {
                    origin,
                    stage: Stage::Draft(validation.draft, validation.target),
                });
                self.error(&e);
            }
        }
    }
    pub(super) fn cancel_capture_validation(&mut self) -> Action {
        let Some(state) = self.capture.take() else {
            return Action::Wait;
        };
        let Stage::Finalizing(validation) = state.stage else {
            unreachable!("owned finalization cancellation")
        };
        self.org.cancel_capture();
        self.capture = Some(State {
            origin: state.origin,
            stage: Stage::Draft(validation.draft, validation.target),
        });
        self.message("Capture validation cancelled; draft retained")
    }
    pub(in crate::session) fn finish_capture_finalize(
        &mut self,
        insertion: Result<Insertion, String>,
    ) {
        let Some(state) = self.capture.take() else {
            return;
        };
        let Stage::Finalizing(validation) = state.stage else {
            self.capture = Some(state);
            return;
        };
        let result = insertion.and_then(|insertion| {
            self.check_capture_draft(validation.stamp)?;
            self.publish_capture(insertion, state.origin.view)
        });
        match result {
            Ok(applied) => {
                self.capture_publication(state.origin, *validation, applied);
            }
            Err(e) => {
                self.capture = Some(State {
                    origin: state.origin,
                    stage: Stage::Draft(validation.draft, validation.target),
                });
                self.error(&format!("Capture not inserted; draft retained: {e}"));
            }
        }
    }
    fn check_capture_draft(&self, stamp: Stamp) -> Result<(), String> {
        if !self
            .buffers
            .get(stamp.buffer)
            .is_ok_and(|b| b.stamp() == stamp && b.is_transient())
        {
            return Err("Capture draft changed during validation".into());
        }
        Ok(())
    }
    fn capture_publication(
        &mut self,
        origin: Origin,
        validation: Validation,
        applied: Applied,
    ) -> Action {
        match applied.outcome.publication() {
            Publication::NotPublished => {
                self.capture = Some(State {
                    origin,
                    stage: Stage::Draft(validation.draft, validation.target),
                });
                self.error(&format!(
                    "Capture not inserted; draft retained: {}",
                    applied.outcome.error().expect("rejected write error")
                ))
            }
            Publication::Complete => self.return_capture(origin, validation.draft),
            Publication::Uncertain => {
                let stored = Stored {
                    draft: validation.draft,
                    draft_stamp: validation.stamp,
                    source: applied.source,
                    path: applied.outcome.target().to_owned(),
                    target: validation.target,
                };
                self.capture = Some(State {
                    origin,
                    stage: Stage::Published(Box::new(stored)),
                });
                self.error("Capture inserted once; save completion unresolved; draft retained; C-c C-c retries save")
            }
        }
    }
}

#[cfg(test)]
mod tests;
