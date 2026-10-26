use crate::buffer::{Buffer, Span};
use crate::coordinates::{Coordinates, Scalar};
use crate::display::Frame;
use crate::input::SearchAction;
use crate::search::{Error, Pattern};
use crate::view::View;
use crate::window::ViewId;

use super::{Action, Pending, Session};

pub(super) struct State {
    origin: View,
    view: ViewId,
    anchor: Scalar,
    found: Option<Span>,
}

impl Session {
    /// Returns only a match qualified for the selected live source revision.
    pub fn search_match(&self) -> Option<&Span> {
        let Some(Pending::Search(state)) = &self.pending else {
            return None;
        };
        self.search_valid(state)
            .then_some(state.found.as_ref())
            .flatten()
    }

    fn search_valid(&self, state: &State) -> bool {
        state.view == self.windows.selected_id()
            && state.origin.point.stamp == self.windows.selected().point.stamp
    }

    pub(super) fn invalidate_search(&mut self) -> bool {
        let invalid = match &self.pending {
            Some(Pending::Search(state)) => !self.search_valid(state),
            _ => false,
        };
        if invalid {
            self.pending = None;
            self.input.cancel();
            self.message("Search source changed");
        }
        invalid
    }

    pub(super) fn search_forward(&mut self) -> Action {
        if matches!(self.pending, Some(Pending::Search(_))) {
            return self.search_step(true);
        }
        if let Err(error) = self.input.search() {
            return self.message(&error.to_string());
        }
        let origin = self.windows.selected().clone();
        self.pending = Some(Pending::Search(Box::new(State {
            anchor: origin.point.position,
            origin,
            view: self.windows.selected_id(),
            found: None,
        })));
        self.message("Type a query; C-s finds next")
    }

    pub(super) fn search_step(&mut self, next: bool) -> Action {
        if self.invalidate_search() {
            return Action::Redraw;
        }
        let Some(prompt) = self.input.prompt() else {
            return self.message("No search is active");
        };
        let Some(Pending::Search(state)) = &mut self.pending else {
            return self.message("No search is active");
        };
        let buffer = match self.buffers.get(state.origin.point.stamp.buffer) {
            Ok(buffer) => buffer,
            Err(error) => return self.message(&error.to_string()),
        };
        let position = match state.update(buffer, prompt.text(), next) {
            Ok(position) => position,
            Err(error) => return self.message(&error.to_string()),
        };
        if let Some(position) = position {
            let view = self.windows.selected_mut();
            view.point.position = position;
            view.reset_navigation();
            return self.message("");
        }
        self.message("No match")
    }

    pub(super) fn search_action(&mut self, action: SearchAction) -> Action {
        if self.invalidate_search() {
            return Action::Redraw;
        }
        match action {
            SearchAction::Next => self.search_step(true),
            SearchAction::Accept => {
                self.pending = None;
                self.message("")
            }
        }
    }

    pub(super) fn restore_search(&mut self) {
        if let Some(Pending::Search(state)) = &self.pending
            && self.search_valid(state)
        {
            *self.windows.selected_mut() = state.origin.clone();
        }
    }

    pub(super) fn highlight_search(&self, frame: &mut Frame) -> Result<(), crate::view::Error> {
        if let Some(span) = self.search_match() {
            frame.search(
                span,
                self.buffer()?,
                self.view(),
                self.windows.area(self.windows.selected_id(), self.size),
            )?;
        }
        Ok(())
    }
}

impl State {
    fn update(
        &mut self,
        buffer: &Buffer,
        query: &str,
        next: bool,
    ) -> Result<Option<Scalar>, Error> {
        if query.is_empty() {
            self.anchor = self.origin.point.position;
            self.found = None;
            return Ok(Some(self.anchor));
        }
        if next && let Some(found) = &self.found {
            self.anchor = Scalar(found.range.end);
        }
        self.found = Pattern::new(query)?.find(buffer, self.anchor)?;
        let Some(found) = &self.found else {
            return Ok(None);
        };
        let coordinates = Coordinates::new(buffer)?;
        let end = Scalar(found.range.end);
        Ok(Some(if coordinates.is_boundary(end)? {
            end
        } else {
            coordinates.next(end)?
        }))
    }
}
