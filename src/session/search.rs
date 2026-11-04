use crate::buffer::{Buffer, Span};
use crate::coordinates::{Coordinates, Scalar};
use crate::display::Frame;
use crate::input::SearchAction;
use crate::search::{Direction, Error, Pattern, Request};
mod continuation;
use crate::view::View;
use crate::window::ViewId;
pub(super) use continuation::Confirmed;

use super::{Action, Pending, Session};

pub(super) struct State {
    origin: View,
    view: ViewId,
    anchor: Scalar,
    found: Option<Span>,
    direction: Direction,
    point: Scalar,
    query: String,
    count: usize,
    operator: Option<crate::command::Editing>,
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

    pub(super) fn search(&mut self, direction: Direction) -> Action {
        if let Some(Pending::Search(state)) = &mut self.pending {
            state.repeat(direction);
            self.input.search_label(direction);
            return self.search_step();
        }
        if let Err(error) = self.input.search(direction) {
            return self.error(&error.to_string());
        }
        let origin = self.windows.selected().clone();
        self.pending = Some(Pending::Search(Box::new(State {
            anchor: origin.point.position,
            point: origin.point.position,
            origin,
            view: self.windows.selected_id(),
            found: None,
            direction,
            query: String::new(),
            count: 1,
            operator: None,
        })));
        self.message("Type a query; C-s/C-r continues")
    }

    pub(super) fn search_step(&mut self) -> Action {
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
            Err(error) => return self.error(&error.to_string()),
        };
        let position = match state.update(buffer, prompt.text()) {
            Ok(position) => position,
            Err(error) => return self.error(&error.to_string()),
        };
        if let Some(position) = position {
            if state.operator.is_some() {
                return self.message("");
            }
            let view = self.windows.selected_mut();
            view.point.position = position;
            view.reset_navigation();
            return self.message("");
        }
        self.message("No match")
    }

    pub(super) fn search_action(&mut self, action: SearchAction) -> Action {
        if let Some(action) = self.terminal_copy_search_action(action) {
            return action;
        }
        if self.invalidate_search() {
            return Action::Redraw;
        }
        match action {
            SearchAction::Repeat(direction) => self.search(direction),
            SearchAction::Accept => {
                if let Some(action) = self.accept_operator_search() {
                    return action;
                }
                if let Some(Pending::Search(state)) = &self.pending
                    && state.found.is_some()
                {
                    let jump = self.prepare_jump_between(
                        state.origin.point,
                        self.buffer().expect("checked search source"),
                        self.view().point,
                    );
                    match jump {
                        Ok(record) => self.publish_jump(record),
                        Err(error) => return self.error(&error.to_string()),
                    }
                }
                self.confirm_search();
                self.message("")
            }
        }
    }

    pub(super) fn accept_operator_search(&mut self) -> Option<Action> {
        let Some(Pending::Search(state)) = &self.pending else {
            return None;
        };
        let mut operator = state.operator?;
        if state.found.is_none() {
            self.kills.break_chain();
            self.pending = None;
            return Some(self.message("Operator search target unavailable"));
        }
        operator.operand = Some(crate::command::editing::Operand::Resolved {
            origin: state.origin.point,
            end: state.point,
        });
        self.confirm_search();
        Some(self.execute(crate::command::Command::Edit(operator)))
    }

    pub(super) fn operator_search(&mut self, request: crate::command::Editing) -> Option<Action> {
        let Some(crate::command::editing::Operand::Motion(navigation)) = request.operand else {
            return None;
        };
        let crate::command::navigation::Target::Search(search) = navigation.target else {
            return None;
        };
        let direction = match search {
            crate::command::navigation::Search::Forward => Direction::Forward,
            crate::command::navigation::Search::Backward => Direction::Backward,
            _ => return None,
        };
        let action = self.search(direction);
        if let Some(Pending::Search(state)) = &mut self.pending {
            state.operator = Some(request);
            state.count = navigation.amount();
        }
        Some(action)
    }

    pub(super) fn restore_search(&mut self) {
        if let Some(Pending::Search(state)) = &self.pending
            && self.search_valid(state)
        {
            *self.windows.selected_mut() = state.origin.clone();
        }
    }

    pub(super) fn highlight_search(
        &self,
        frame: &mut Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), crate::view::Error> {
        if let Some(span) = self.search_match().or_else(|| self.replace_match()) {
            frame.search(
                span,
                self.buffer()?,
                self.view(),
                composition.and_then(|composition| {
                    composition
                        .panels()
                        .iter()
                        .find(|(_, panel)| panel.active())
                        .map(|(_, panel)| *panel)
                }),
            )?;
        }
        Ok(())
    }
}

impl State {
    fn repeat(&mut self, direction: Direction) {
        if let Some(found) = &self.found {
            self.anchor = match direction {
                Direction::Forward => Scalar(found.range.end),
                Direction::Backward => Scalar(found.range.start),
            };
        } else if direction != self.direction {
            self.anchor = self.point;
        }
        self.direction = direction;
    }

    fn update(&mut self, buffer: &Buffer, query: &str) -> Result<Option<Scalar>, Error> {
        if query.is_empty() {
            self.query.clear();
            self.anchor = self.origin.point.position;
            self.found = None;
            self.point = self.anchor;
            return Ok(Some(self.anchor));
        }
        let pattern = Pattern::new(query)?;
        self.query = query.into();
        self.found = pattern.occurrence(
            buffer,
            Request {
                anchor: self.anchor,
                direction: self.direction,
                count: self.count,
                whole_word: false,
            },
        )?;
        let Some(found) = &self.found else {
            return Ok(None);
        };
        let coordinates = Coordinates::new(buffer)?;
        self.point = match self.direction {
            Direction::Forward => snap(&coordinates, Scalar(found.range.end), true)?,
            Direction::Backward => snap(&coordinates, Scalar(found.range.start), false)?,
        };
        Ok(Some(self.point))
    }
}

fn snap(coordinates: &Coordinates<'_>, point: Scalar, forward: bool) -> Result<Scalar, Error> {
    if coordinates.is_boundary(point)? {
        return Ok(point);
    }
    Ok(if forward {
        coordinates.next(point)?
    } else {
        coordinates.previous(point)?
    })
}
