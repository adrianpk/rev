use crate::command::{Motion, Scroll};
use crate::coordinates::Coordinates;
use crate::view::Error;

use super::{Action, Session};

impl Session {
    pub(super) fn scroll(&mut self, scroll: Scroll) -> Action {
        match self.scroll_point(scroll) {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn scroll_point(&mut self, scroll: Scroll) -> Result<(), Error> {
        let buffer = self
            .buffers
            .get(self.windows.selected().point.stamp.buffer)?;
        let mut candidate = self.windows.selected().clone();
        let size = self.pane_size();
        if scroll == Scroll::Recenter {
            candidate.recenter(buffer, size)?;
        } else {
            let overlap = self.buffers.snapshot().settings().scroll.page_overlap;
            candidate.scroll(buffer, scroll, distance(scroll, size, overlap))?;
        }
        candidate.reveal(buffer, self.pane_size())?;
        *self.windows.selected_mut() = candidate;
        Ok(())
    }

    pub(super) fn motion(&mut self, motion: Motion) -> Action {
        match self.move_point(motion) {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn move_point(&mut self, motion: Motion) -> Result<(), Error> {
        let buffer = self
            .buffers
            .get(self.windows.selected().point.stamp.buffer)?;
        self.windows.selected().cell(buffer)?;
        let coordinates = Coordinates::new(buffer)?;
        let mut candidate = self.windows.selected().clone();
        candidate.point.position = motion.target(&coordinates, candidate.point.position)?;
        candidate.reveal(buffer, self.pane_size())?;
        *self.windows.selected_mut() = candidate;
        Ok(())
    }
}

// The global overlap is clamped against current content rows, never against the
// document. Tiny usable panes still advance; suspended pages retain their point.
fn distance(scroll: Scroll, size: crate::view::Size, overlap: usize) -> usize {
    if !scroll.page() {
        return 1;
    }
    if !size.usable() {
        return 0;
    }
    usize::from(size.text_rows()).saturating_sub(overlap).max(1)
}
