use std::time::Instant;

use crate::buffer::edit::{Error, Point};
use crate::command::Command;
use crate::coordinates::{Coordinates, Scalar};
use crate::input::EditingState;
use crate::pairs::{GroupOutcome, GroupRequest, Outcome, Status, Ticket};
use crate::session::{Action, Session};
use crate::view::View;
use crate::window::ViewId;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Intent {
    Mate,
    Groups,
}

/// One disposable source motion; all qualified facts remain buffer-owned.
pub(in crate::session) struct Navigation {
    intent: Intent,
    point: Point,
    view: ViewId,
    mark: (Option<Scalar>, bool),
    state: Option<EditingState>,
    ticket: Ticket,
    candidate: Option<View>,
}

impl Session {
    pub(in crate::session) fn matching_delimiter(&mut self) -> Action {
        self.begin_pair_navigation(None)
    }

    pub(in crate::session) fn group_motion(&mut self, command: Command, count: i32) -> Action {
        if count == 0 {
            return self.message("");
        }
        if !self.pair_navigation_focus() {
            return self.error("Group motion requires source input focus");
        }
        self.command_boundary(command);
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        let request = self.group_request(command, count);
        self.begin_pair_navigation(Some(request))
    }

    fn group_request(&self, command: Command, count: i32) -> GroupRequest {
        let point = self.view().point.position;
        match command {
            Command::ForwardPair | Command::BackwardPair => {
                GroupRequest::new(point, command == Command::BackwardPair, count)
            }
            Command::UpPair | Command::DownPair => {
                GroupRequest::nesting(point, command == Command::DownPair, count)
            }
            Command::ForwardSexp | Command::BackwardSexp => {
                GroupRequest::expression(point, command == Command::BackwardSexp, count)
            }
            _ => unreachable!("group command admission precedes request construction"),
        }
        .expect("native prefix magnitude is admitted")
    }

    fn begin_pair_navigation(&mut self, groups: Option<GroupRequest>) -> Action {
        if !self.pair_navigation_focus() {
            return self.error("Delimiter navigation requires source input focus");
        }
        let point = self.view().point;
        let ticket =
            match self
                .buffers
                .get_mut(point.stamp.buffer)
                .and_then(|buffer| match groups {
                    Some(request) => buffer.start_group_request(request),
                    None => buffer.start_mate_request(point.position),
                }) {
                Ok(ticket) => ticket,
                Err(error) => return self.error(&error.to_string()),
            };
        let buffer = self.buffer().expect("admitted navigation source");
        self.layout.navigation = Some(Navigation {
            intent: if groups.is_some() {
                Intent::Groups
            } else {
                Intent::Mate
            },
            point,
            view: self.windows.selected_id(),
            mark: (buffer.mark(), buffer.mark_active()),
            state: self.editing_state(),
            ticket,
            candidate: None,
        });
        self.layout_pair_navigation()
            .unwrap_or_else(|| self.publish_pair_navigation())
    }

    fn pair_navigation_focus(&self) -> bool {
        self.source_layout_focus()
            && !self.terminals.focused
            && !self.git_visible()
            && !self.capture_blocks_input()
            && !self.context_storage.locked()
            && self.pending.is_none()
            && !self.input.editing_composition()
    }

    fn pair_navigation_current(&self, work: &Navigation) -> bool {
        self.pair_navigation_focus()
            && work.point == self.view().point
            && work.view == self.windows.selected_id()
            && work.state == self.editing_state()
            && self.buffer().is_ok_and(|buffer| {
                work.ticket.stamp == buffer.stamp()
                    && work.mark == (buffer.mark(), buffer.mark_active())
            })
    }

    pub(in crate::session) fn cancel_pair_navigation(&mut self) {
        if let Some(work) = self.layout.navigation.take()
            && let Ok(buffer) = self.buffers.get_mut(work.ticket.stamp.buffer)
        {
            buffer.cancel_pair_request(work.ticket);
        }
    }

    pub(in crate::session) fn prepare_pair_navigation(&mut self) -> Result<(), crate::view::Error> {
        let mut work = self
            .layout
            .navigation
            .take()
            .expect("owned delimiter navigation request");
        let result = self.prepare_pair_navigation_target(&mut work);
        self.layout.navigation = Some(work);
        result
    }

    fn prepare_pair_navigation_target(
        &mut self,
        work: &mut Navigation,
    ) -> Result<(), crate::view::Error> {
        if !self.pair_navigation_current(work) {
            return Err(Error::ChangedPositions.into());
        }
        let buffer = self.buffers.get_mut(work.ticket.stamp.buffer)?;
        match buffer.poll_pair_request(work.ticket, Instant::now() + crate::pairs::SLICE_TIME) {
            Status::Pending => return Err(crate::coordinates::Error::Pending.into()),
            Status::Retired => return Err(Error::ChangedPositions.into()),
            Status::Ready => (),
        }
        if let Some(target) = self.pair_navigation_target(work) {
            work.candidate = Some(self.pair_navigation_view(target)?);
        }
        Ok(())
    }

    fn pair_navigation_target(&self, work: &Navigation) -> Option<Scalar> {
        let buffer = self.buffer().ok()?;
        if work.intent == Intent::Groups {
            return match buffer.group_answer(work.ticket)?.outcome {
                GroupOutcome::Target(point) | GroupOutcome::Boundary(point) => Some(point),
                _ => None,
            };
        }
        let answer = buffer.pair_answer(work.ticket)?;
        let Outcome::Confirmed(pair) = &answer.outcome else {
            return None;
        };
        Some(Scalar(
            if answer.query.point().0 == pair.opening.range.start {
                pair.closing.range.start
            } else {
                pair.opening.range.start
            },
        ))
    }

    fn pair_navigation_view(&self, target: Scalar) -> Result<View, crate::view::Error> {
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

    pub(in crate::session) fn publish_pair_navigation(&mut self) -> Action {
        let Some(work) = self.layout.navigation.take() else {
            return self.message("Delimiter navigation request retired");
        };
        if !self.pair_navigation_current(&work) {
            return self.message("Delimiter navigation source changed");
        }
        let Some(candidate) = work.candidate else {
            return self.error(if work.intent == Intent::Mate {
                "No confirmed matching delimiter at or immediately before point"
            } else {
                "No qualified group or expression in that direction"
            });
        };
        let result = if work.intent == Intent::Mate {
            self.place_jump_view(candidate)
        } else {
            self.place_view(candidate)
        };
        self.sync_input_context();
        match result {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }
}
