pub(super) mod find;
mod viewport;

use crate::command::navigation::Target;
use crate::command::{Command, Navigation, Scroll};
use crate::coordinates::{Cell, Coordinates, Scalar};
use crate::search::Direction;
use crate::view::{Error, View};

use super::{Action, Session};

impl Session {
    pub(super) fn navigation_dispatch(&mut self, command: Command) -> Option<Action> {
        let Command::Navigate(request) = command else {
            return None;
        };
        if let Target::Search(search) = request.target {
            return Some(self.navigation_search(search, request.amount()));
        }
        if let Some(action) = self.navigation_argument(request) {
            return Some(action);
        }
        Some(match self.apply_navigation(request) {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        })
    }

    fn apply_navigation(&mut self, request: Navigation) -> Result<(), Error> {
        let found = self.navigation_find(request)?;
        let candidate = self.navigation_view(request)?;
        self.place_view(candidate)?;
        self.retain_find(request, found);
        Ok(())
    }

    pub(super) fn navigation_view(&self, request: Navigation) -> Result<View, Error> {
        let buffer = self.buffer()?;
        let mut candidate = self.view().clone();
        candidate.check_point(buffer)?;
        if self.navigation_viewport(&mut candidate, request)? {
            candidate.reveal(buffer, self.pane_size())?;
            return Ok(candidate);
        }
        if let Some(point) = self.navigation_position(request)? {
            candidate.point.position = point;
        }
        candidate.reset_navigation();
        candidate.check_point(buffer)?;
        candidate.reveal(buffer, self.pane_size())?;
        Ok(candidate)
    }

    fn navigation_position(&self, request: Navigation) -> Result<Option<Scalar>, Error> {
        if matches!(request.target, Target::Find(_) | Target::RepeatFind(_)) {
            return Ok(self.navigation_find(request)?.map(|found| found.point));
        }
        self.navigation_source(request).map(Some)
    }

    fn navigation_source(&self, request: Navigation) -> Result<Scalar, Error> {
        let coordinates = Coordinates::new(self.buffer()?)?;
        let point = self.view().point.position;
        match request.target {
            Target::Buffer(direction) => {
                self.navigation_line(&coordinates, direction, request.count)
            }
            Target::LineEnd => self.counted_line_end(request.amount()),
            _ => Ok(coordinates.navigation(point, request.target, request.amount())?),
        }
    }

    fn navigation_line(
        &self,
        coordinates: &Coordinates<'_>,
        direction: Direction,
        count: Option<u32>,
    ) -> Result<Scalar, Error> {
        let Some(count) = count else {
            return Ok(if direction == Direction::Backward {
                Scalar(0)
            } else {
                coordinates.end()
            });
        };
        let row = (count.max(1) as usize - 1).min(self.buffer()?.len_lines() - 1);
        Ok(coordinates.from_cell(Cell { row, column: 0 })?)
    }

    fn counted_line_end(&self, count: usize) -> Result<Scalar, Error> {
        let mut candidate = self.view().clone();
        candidate.scroll(self.buffer()?, Scroll::NextLine, count.saturating_sub(1))?;
        Ok(Coordinates::new(self.buffer()?)?.line_end(candidate.point.position)?)
    }
}
