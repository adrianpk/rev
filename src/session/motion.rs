use crate::command::{Motion, Scroll};
use crate::coordinates::Coordinates;
use crate::view::Error;

use super::{Action, Session};

impl Session {
    pub(super) fn scroll(&mut self, scroll: Scroll) -> Action {
        match self.scroll_point(scroll) {
            Ok(()) => self.message(""),
            Err(error) => self.message(&error.to_string()),
        }
    }

    fn scroll_point(&mut self, scroll: Scroll) -> Result<(), Error> {
        let buffer = self.buffers.get(self.view.point.stamp.buffer)?;
        let mut candidate = self.view.clone();
        candidate.scroll(buffer, self.size, scroll)?;
        candidate.reveal(buffer, self.size)?;
        self.view = candidate;
        Ok(())
    }

    pub(super) fn motion(&mut self, motion: Motion) -> Action {
        match self.move_point(motion) {
            Ok(()) => self.message(""),
            Err(error) => self.message(&error.to_string()),
        }
    }

    fn move_point(&mut self, motion: Motion) -> Result<(), Error> {
        let buffer = self.buffers.get(self.view.point.stamp.buffer)?;
        self.view.cell(buffer)?;
        let coordinates = Coordinates::new(buffer)?;
        let mut candidate = self.view.clone();
        candidate.point.position = motion.target(&coordinates, candidate.point.position)?;
        candidate.reveal(buffer, self.size)?;
        self.view = candidate;
        Ok(())
    }
}
