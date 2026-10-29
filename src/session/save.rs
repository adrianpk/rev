use std::path::Path;

use crate::buffer::{BufferId, Confirmation, Kill, SaveRequest};
use crate::file::{Outcome, Publication};

use super::{Action, AfterSave, Pending, Session};

impl Session {
    pub(super) fn save_buffer(&mut self) -> Action {
        match self.buffer() {
            Ok(buffer) if buffer.file().is_none() => self.write_file(),
            Ok(buffer) => self.save_to(buffer.id(), None, AfterSave::Stay),
            Err(error) => self.error(&error.to_string()),
        }
    }

    pub(super) fn write_file(&mut self) -> Action {
        self.write_argument(self.windows.selected().point.stamp.buffer, AfterSave::Stay)
    }

    pub(super) fn write_argument(&mut self, buffer: BufferId, after: AfterSave) -> Action {
        if let Err(error) = self.input.argument("Write file: ", "") {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::Write { buffer, after });
        self.message("")
    }

    pub(super) fn save_to(
        &mut self,
        id: BufferId,
        target: Option<&Path>,
        after: AfterSave,
    ) -> Action {
        match self.buffers.request_save(id, target) {
            Ok(SaveRequest::Ready(save)) => {
                let state = save.state();
                let outcome = self.buffers.publish_save(save);
                self.saved_state(state, &outcome, after)
            }
            Ok(SaveRequest::Confirm(confirmation)) => self.confirm(confirmation, after),
            Err(error) => {
                self.note_refile_rejection(id, &error.to_string());
                self.error(&format!("save: {error}"))
            }
        }
    }

    fn confirm(&mut self, confirmation: Confirmation, after: AfterSave) -> Action {
        if let Err(error) = self.input.decision("Save? [y/n]") {
            return self.error(&error.to_string());
        }
        self.echo_kind = crate::display::Message::Normal;
        self.echo = confirmation.message().into();
        self.pending = Some(Pending::Save {
            confirmation: Box::new(confirmation),
            after,
        });
        Action::Redraw
    }

    pub(super) fn decision(&mut self, accepted: bool) -> Action {
        match self.pending.take() {
            Some(Pending::Quit(quit)) => self.quit_answer(quit, accepted),
            Some(Pending::TreeDelete(deletion)) => self.delete_answer(deletion, accepted),
            Some(Pending::Replace(state)) => self.replace_answer(*state, accepted),
            Some(Pending::Kill { kill, .. }) => {
                if accepted {
                    self.save_killing(kill)
                } else {
                    self.close_buffer(kill)
                }
            }
            Some(Pending::Save {
                confirmation,
                after,
            }) => {
                if !accepted {
                    return self.message("Save canceled");
                }
                let state = confirmation.state();
                let outcome = self.buffers.confirm_save(*confirmation);
                self.saved_state(state, &outcome, after)
            }
            _ => self.message("No decision is pending"),
        }
    }

    pub(super) fn saved_state(
        &mut self,
        state: crate::buffer::State,
        outcome: &Outcome,
        after: AfterSave,
    ) -> Action {
        self.note_refile_save(state, outcome);
        self.saved(outcome, after)
    }

    pub(super) fn saved(&mut self, outcome: &Outcome, after: AfterSave) -> Action {
        let refreshed = if outcome.publication() != Publication::NotPublished {
            self.tree.observe_path(outcome.target())
        } else {
            Ok(())
        };
        if outcome.is_complete()
            && let Err(error) = self.checkpoint_saved_clock(outcome.target())
        {
            return self.error(&format!("Source saved; {error}"));
        }
        let action = self.after_saved(outcome, after);
        if let Err(error) = refreshed {
            self.echo
                .push_str(&format!("; tree refresh failed: {error}"));
        }
        action
    }

    fn after_saved(&mut self, outcome: &Outcome, after: AfterSave) -> Action {
        if !outcome.is_complete() {
            return self.save_result(outcome);
        }
        match after {
            AfterSave::Stay => self.save_result(outcome),
            AfterSave::Close(kill) => self.close_saved(kill),
            AfterSave::Quit(quit) => self.quit_saved(quit),
        }
    }

    fn close_saved(&mut self, kill: Kill) -> Action {
        let buffer = match self.buffers.get(kill.id()) {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        if buffer.is_modified() {
            return self.message("Buffer changed; close canceled");
        }
        self.close_buffer(kill)
    }

    pub(super) fn focus_message(&mut self, message: &str) -> Action {
        self.echo_kind = crate::display::Message::Normal;
        if let Some(Pending::TreeDelete(deletion)) = &self.pending {
            self.echo_kind = crate::display::Message::Normal;
            self.echo = format!("{}; {message}", deletion.message());
            return Action::Redraw;
        }
        let question = match &self.pending {
            Some(Pending::Replace(state)) if state.deciding() => {
                Some("Replace occurrence? [y/n/q]")
            }
            Some(Pending::Quit(quit)) => Some(quit.question.as_str()),
            Some(Pending::Save { confirmation, .. }) => Some(confirmation.message()),
            Some(Pending::Kill { message, .. }) => Some(message.as_str()),
            _ => None,
        };
        if let Some(question) = question {
            self.echo_kind = crate::display::Message::Normal;
            self.echo = format!("{question}; {message}");
            return Action::Redraw;
        }
        self.message(message)
    }

    fn save_result(&mut self, outcome: &Outcome) -> Action {
        if outcome.is_complete() {
            return self.message(&format!("Saved {:?}", outcome.target()));
        }
        let publication = match outcome.publication() {
            Publication::NotPublished => "not published",
            _ => "published; completion unresolved",
        };
        let error = outcome.error().map(ToString::to_string).unwrap_or_default();
        self.error(&format!(
            "save: {publication}: {error}; target {:?}",
            outcome.target()
        ))
    }
}
