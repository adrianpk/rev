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
        let candidate = self.scrolling_view(scroll)?;
        *self.windows.selected_mut() = candidate;
        Ok(())
    }

    pub(super) fn scrolling_view(&self, scroll: Scroll) -> Result<crate::view::View, Error> {
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
                candidate.visual_scroll(buffer, width, scroll, distance(scroll, size, overlap))?;
            } else {
                candidate.scroll(buffer, scroll, distance(scroll, size, overlap))?;
            }
        }
        candidate.reveal(buffer, self.pane_size())?;
        Ok(candidate)
    }

    pub(super) fn motion(&mut self, motion: Motion) -> Action {
        match self.move_point(motion) {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn move_point(&mut self, motion: Motion) -> Result<(), Error> {
        let target = self.motion_target(motion)?;
        let mut candidate = self.view().clone();
        candidate.point.position = target;
        candidate.check_source(self.buffer()?)?;
        candidate.reset_navigation();
        self.place_view(candidate)
    }

    pub(super) fn motion_target(
        &self,
        motion: Motion,
    ) -> Result<crate::coordinates::Scalar, Error> {
        let buffer = self
            .buffers
            .get(self.windows.selected().point.stamp.buffer)?;
        self.windows.selected().check_source(buffer)?;
        let coordinates = Coordinates::new(buffer)?;
        let width = self.view().text_width(buffer, self.pane_size())?;
        if motion == Motion::ForwardChar {
            return Ok(coordinates.next_observed(self.view().point.position)?);
        }
        if motion == Motion::BackwardChar {
            return Ok(coordinates.previous_observed(self.view().point.position)?);
        }
        let target = if buffer.soft_wrap()
            && width > 0
            && matches!(motion, Motion::LineStart | Motion::LineEnd)
        {
            self.view()
                .visual_boundary(buffer, width, motion == Motion::LineEnd)?
        } else {
            motion.target(&coordinates, self.view().point.position)?
        };
        Ok(target)
    }

    pub(super) fn place_point(&mut self, point: crate::buffer::edit::Point) -> Result<(), Error> {
        let buffer = self.buffer()?;
        let mut candidate = self.windows.selected().clone();
        candidate.point = point;
        candidate.check_point(buffer)?;
        candidate.reset_navigation();
        self.place_view(candidate)
    }

    pub(super) fn place_view(&mut self, candidate: crate::view::View) -> Result<(), Error> {
        let buffer = self.buffer()?;
        let work = std::sync::Arc::new(std::sync::Mutex::new(crate::view::LayoutBudget::new()));
        let coordinates = crate::coordinates::Scope::new(buffer, work);
        let fragments = candidate.layout_scope(coordinates.work());
        let mut revealed = candidate.clone();
        let result = revealed.reveal(buffer, self.pane_size());
        drop(fragments);
        drop(coordinates);
        let candidate = match result {
            Ok(()) => revealed,
            Err(error) if error.pending() => {
                self.layout_pending = true;
                candidate
            }
            Err(error) => return Err(error),
        };
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
