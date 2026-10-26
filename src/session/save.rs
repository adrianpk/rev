use crate::file::{Outcome, Publication};

use super::{Action, Session};

impl Session {
    pub(super) fn save_buffer(&mut self) -> Action {
        let id = self.view.point.stamp.buffer;
        match self.buffers.save(id) {
            Ok(outcome) => self.save_result(&outcome),
            Err(error) => self.message(&format!("save: {error}")),
        }
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
            "save {:?}: {publication}: {error}",
            outcome.target()
        ))
    }
}
