use std::path::Path;

use crate::buffer::{Confirmation, SaveRequest};
use crate::command::Command;
use crate::file::{Outcome, Publication};

use super::{Action, Pending, Session};

impl Session {
    pub(super) fn save_buffer(&mut self) -> Action {
        match self.buffer() {
            Ok(buffer) if buffer.file().is_none() => self.write_file(),
            Ok(_) => self.save_to(None),
            Err(error) => self.message(&error.to_string()),
        }
    }

    pub(super) fn write_file(&mut self) -> Action {
        self.file_argument(Command::WriteFile, "Write file: ")
    }

    pub(super) fn save_to(&mut self, target: Option<&Path>) -> Action {
        let id = self.view.point.stamp.buffer;
        match self.buffers.request_save(id, target) {
            Ok(SaveRequest::Ready(save)) => {
                let outcome = self.buffers.publish_save(save);
                self.save_result(&outcome)
            }
            Ok(SaveRequest::Confirm(confirmation)) => self.confirm(confirmation),
            Err(error) => self.message(&format!("save: {error}")),
        }
    }

    fn confirm(&mut self, confirmation: Confirmation) -> Action {
        if let Err(error) = self.input.decision("Save? [y/n]") {
            return self.message(&error.to_string());
        }
        self.echo = confirmation.message().into();
        self.pending = Some(Pending::Save(Box::new(confirmation)));
        Action::Redraw
    }

    pub(super) fn save_decision(&mut self, accepted: bool) -> Action {
        let Some(Pending::Save(confirmation)) = self.pending.take() else {
            return self.message("No save decision is pending");
        };
        if !accepted {
            return self.message("Save canceled");
        }
        let outcome = self.buffers.confirm_save(*confirmation);
        self.save_result(&outcome)
    }

    pub(super) fn focus_message(&mut self, message: &str) -> Action {
        if let Some(Pending::Save(confirmation)) = &self.pending {
            self.echo = format!("{}; {message}", confirmation.message());
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
