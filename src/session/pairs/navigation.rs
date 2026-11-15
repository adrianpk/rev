use std::time::Instant;

use crate::buffer::edit::{Error, Point};
use crate::coordinates::{Coordinates, Scalar};
use crate::input::EditingState;
use crate::pairs::{Outcome, Status, Ticket};
use crate::session::{Action, Session};
use crate::view::View;
use crate::window::ViewId;

#[cfg(test)]
mod tests;

/// One disposable mate operation; all lexical facts remain buffer-owned.
pub(in crate::session) struct Mate {
    point: Point,
    view: ViewId,
    mark: (Option<Scalar>, bool),
    state: Option<EditingState>,
    ticket: Ticket,
    candidate: Option<View>,
}

impl Session {
    pub(in crate::session) fn matching_delimiter(&mut self) -> Action {
        if !self.mate_focus() {
            return self.error("Matching delimiter requires source input focus");
        }
        let point = self.view().point;
        let ticket = match self
            .buffers
            .get_mut(point.stamp.buffer)
            .and_then(|buffer| buffer.start_mate_request(point.position))
        {
            Ok(ticket) => ticket,
            Err(error) => return self.error(&error.to_string()),
        };
        let buffer = self.buffer().expect("admitted mate source");
        self.layout.mate = Some(Mate {
            point,
            view: self.windows.selected_id(),
            mark: (buffer.mark(), buffer.mark_active()),
            state: self.editing_state(),
            ticket,
            candidate: None,
        });
        self.layout_mate().unwrap_or_else(|| self.publish_mate())
    }

    fn mate_focus(&self) -> bool {
        self.source_layout_focus()
            && !self.terminals.focused
            && !self.git_visible()
            && !self.capture_blocks_input()
            && !self.context_storage.locked()
            && self.pending.is_none()
            && !self.input.editing_composition()
    }

    fn mate_current(&self, work: &Mate) -> bool {
        self.mate_focus()
            && work.point == self.view().point
            && work.view == self.windows.selected_id()
            && work.state == self.editing_state()
            && self.buffer().is_ok_and(|buffer| {
                work.ticket.stamp == buffer.stamp()
                    && work.mark == (buffer.mark(), buffer.mark_active())
            })
    }

    pub(in crate::session) fn cancel_mate(&mut self) {
        if let Some(work) = self.layout.mate.take()
            && let Ok(buffer) = self.buffers.get_mut(work.ticket.stamp.buffer)
        {
            buffer.cancel_pair_request(work.ticket);
        }
    }

    pub(in crate::session) fn prepare_mate(&mut self) -> Result<(), crate::view::Error> {
        let mut work = self.layout.mate.take().expect("owned mate request");
        let result = self.prepare_mate_target(&mut work);
        self.layout.mate = Some(work);
        result
    }

    fn prepare_mate_target(&mut self, work: &mut Mate) -> Result<(), crate::view::Error> {
        if !self.mate_current(work) {
            return Err(Error::ChangedPositions.into());
        }
        let buffer = self.buffers.get_mut(work.ticket.stamp.buffer)?;
        match buffer.poll_pair_request(work.ticket, Instant::now() + crate::pairs::SLICE_TIME) {
            Status::Pending => return Err(crate::coordinates::Error::Pending.into()),
            Status::Retired => return Err(Error::ChangedPositions.into()),
            Status::Ready => (),
        }
        let answer = buffer.pair_answer(work.ticket).expect("ready mate proof");
        let Outcome::Confirmed(pair) = &answer.outcome else {
            return Ok(());
        };
        let target = if answer.query.point().0 == pair.opening.range.start {
            pair.closing.range.start
        } else {
            pair.opening.range.start
        };
        work.candidate = Some(self.mate_view(Scalar(target))?);
        Ok(())
    }

    fn mate_view(&self, target: Scalar) -> Result<View, crate::view::Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let mut candidate = self.view().clone();
        candidate.point.position = if coordinates.is_boundary(target)? {
            target
        } else if target > candidate.point.position {
            coordinates.next(target)?
        } else {
            coordinates.previous(target)?
        };
        candidate.reset_navigation();
        candidate.check_point(buffer)?;
        candidate.reveal(buffer, self.pane_size())?;
        Ok(candidate)
    }

    pub(in crate::session) fn publish_mate(&mut self) -> Action {
        let Some(work) = self.layout.mate.take() else {
            return self.message("Matching delimiter request retired");
        };
        if !self.mate_current(&work) {
            return self.message("Matching delimiter source changed");
        }
        let Some(candidate) = work.candidate else {
            return self.error("No confirmed matching delimiter at or immediately before point");
        };
        let result = self.place_jump_view(candidate);
        self.sync_input_context();
        match result {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }
}
