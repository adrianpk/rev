use std::path::Path;

use crate::buffer::{BufferId, Confirmation, Kill, SaveRequest};
use crate::file::{Outcome, Publication};

use super::{Action, Pending, Session};

impl Session {
    pub(super) fn save_buffer(&mut self) -> Action {
        match self.buffer() {
            Ok(buffer) if buffer.file().is_none() => self.write_file(),
            Ok(buffer) => self.save_to(buffer.id(), None, None),
            Err(error) => self.message(&error.to_string()),
        }
    }

    pub(super) fn write_file(&mut self) -> Action {
        self.write_argument(self.windows.selected().point.stamp.buffer, None)
    }

    pub(super) fn write_argument(&mut self, buffer: BufferId, close: Option<Kill>) -> Action {
        if let Err(error) = self.input.argument("Write file: ", "") {
            return self.message(&error.to_string());
        }
        self.pending = Some(Pending::Write { buffer, close });
        self.message("")
    }

    pub(super) fn save_to(
        &mut self,
        id: BufferId,
        target: Option<&Path>,
        close: Option<Kill>,
    ) -> Action {
        match self.buffers.request_save(id, target) {
            Ok(SaveRequest::Ready(save)) => {
                let outcome = self.buffers.publish_save(save);
                self.saved(&outcome, close)
            }
            Ok(SaveRequest::Confirm(confirmation)) => self.confirm(confirmation, close),
            Err(error) => self.message(&format!("save: {error}")),
        }
    }

    fn confirm(&mut self, confirmation: Confirmation, close: Option<Kill>) -> Action {
        if let Err(error) = self.input.decision("Save? [y/n]") {
            return self.message(&error.to_string());
        }
        self.echo = confirmation.message().into();
        self.pending = Some(Pending::Save {
            confirmation: Box::new(confirmation),
            close,
        });
        Action::Redraw
    }

    pub(super) fn decision(&mut self, accepted: bool) -> Action {
        match self.pending.take() {
            Some(Pending::Kill { kill, .. }) => {
                if accepted {
                    self.save_killing(kill)
                } else {
                    self.close_buffer(kill)
                }
            }
            Some(Pending::Save {
                confirmation,
                close,
            }) => {
                if !accepted {
                    return self.message("Save canceled");
                }
                let outcome = self.buffers.confirm_save(*confirmation);
                self.saved(&outcome, close)
            }
            _ => self.message("No decision is pending"),
        }
    }

    fn saved(&mut self, outcome: &Outcome, close: Option<Kill>) -> Action {
        if outcome.is_complete()
            && let Some(kill) = close
        {
            let buffer = match self.buffers.get(kill.id()) {
                Ok(buffer) => buffer,
                Err(error) => return self.message(&error.to_string()),
            };
            if buffer.is_modified() {
                return self.message("Buffer changed; close canceled");
            }
            return self.close_buffer(kill);
        }
        self.save_result(outcome)
    }

    pub(super) fn focus_message(&mut self, message: &str) -> Action {
        let question = match &self.pending {
            Some(Pending::Save { confirmation, .. }) => Some(confirmation.message()),
            Some(Pending::Kill { message, .. }) => Some(message.as_str()),
            _ => None,
        };
        if let Some(question) = question {
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
        self.message(&format!(
            "save: {publication}: {error}; target {:?}",
            outcome.target()
        ))
    }
}
