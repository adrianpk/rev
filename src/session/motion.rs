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
            let width = candidate.text_width(buffer, size)?;
            if buffer.soft_wrap() && width > 0 {
                candidate.visual_scroll(
                    buffer,
                    width,
                    scroll.backward(),
                    distance(scroll, size, overlap),
                )?;
            } else {
                candidate.scroll(buffer, scroll, distance(scroll, size, overlap))?;
            }
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
        let width = self.view().text_width(buffer, self.pane_size())?;
        let target = if buffer.soft_wrap()
            && width > 0
            && matches!(motion, Motion::LineStart | Motion::LineEnd)
        {
            self.view()
                .visual_boundary(buffer, width, motion == Motion::LineEnd)?
        } else {
            motion.target(&coordinates, self.view().point.position)?
        };
        self.place_point(crate::buffer::edit::Point {
            position: target,
            ..self.view().point
        })
    }

    pub(super) fn place_point(&mut self, point: crate::buffer::edit::Point) -> Result<(), Error> {
        let buffer = self.buffer()?;
        let mut candidate = self.windows.selected().clone();
        candidate.point = point;
        candidate.cell(buffer)?;
        candidate.reset_navigation();
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
