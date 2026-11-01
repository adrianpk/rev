use crate::buffer::{Span, Stamp};
use crate::command::navigation::Search;
use crate::coordinates::{Coordinates, Scalar};
use crate::search::{Direction, Error, Pattern, Request};
use crate::window::ViewId;

use super::super::{Action, Pending, Session};
use super::snap;

pub(in crate::session) struct Confirmed {
    query: String,
    direction: Direction,
    whole_word: bool,
    view: ViewId,
    stamp: Stamp,
    found: Option<Span>,
}

impl Session {
    pub(in crate::session) fn confirm_search(&mut self) {
        let Some(Pending::Search(state)) = self.pending.take() else {
            return;
        };
        if !state.query.is_empty() {
            self.last_search = Some(Confirmed {
                query: state.query,
                direction: state.direction,
                whole_word: false,
                view: state.view,
                stamp: state.origin.point.stamp,
                found: state.found,
            });
        }
    }

    pub(in crate::session) fn navigation_search(&mut self, search: Search, count: usize) -> Action {
        match search {
            Search::Forward | Search::Backward => {
                let direction = if search == Search::Forward {
                    Direction::Forward
                } else {
                    Direction::Backward
                };
                let action = self.search(direction);
                if let Some(Pending::Search(state)) = &mut self.pending {
                    state.count = count;
                }
                action
            }
            _ => match self.continue_search(search, count) {
                Ok(true) => self.message(""),
                Ok(false) => self.message("No match"),
                Err(error) => self.error(&error.to_string()),
            },
        }
    }

    fn continue_search(&mut self, search: Search, count: usize) -> Result<bool, Error> {
        let word = matches!(search, Search::WordForward | Search::WordBackward);
        if word {
            self.word_search(search)?;
        }
        let Some(confirmed) = &self.last_search else {
            return Ok(false);
        };
        if confirmed.view != self.windows.selected_id()
            || confirmed.stamp != self.view().point.stamp
        {
            return Ok(false);
        }
        let direction = if search == Search::Reverse {
            reverse(confirmed.direction)
        } else {
            confirmed.direction
        };
        let anchor = confirmed.anchor(self.view().point.position, direction);
        let pattern = Pattern::new(&confirmed.query)?;
        let found = pattern.occurrence(
            self.buffer()?,
            Request {
                anchor,
                direction,
                count,
                whole_word: confirmed.whole_word,
            },
        )?;
        let Some(found) = found else { return Ok(false) };
        self.publish_search(found, direction)?;
        Ok(true)
    }

    fn publish_search(&mut self, found: Span, direction: Direction) -> Result<(), Error> {
        let coordinates = Coordinates::new(self.buffer()?)?;
        let target = match direction {
            Direction::Forward => snap(&coordinates, Scalar(found.range.end), true)?,
            Direction::Backward => snap(&coordinates, Scalar(found.range.start), false)?,
        };
        let mut point = self.view().point;
        point.position = target;
        self.place_point(point)?;
        self.last_search
            .as_mut()
            .expect("continuation has a confirmed query")
            .found = Some(found);
        Ok(())
    }

    fn word_search(&mut self, search: Search) -> Result<(), Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let point = self.view().point.position;
        let range = coordinates
            .word_at(point)?
            .ok_or(crate::coordinates::Error::Scalar(point))?;
        let found = buffer.span(range.start.0..range.end.0)?;
        let query: String = buffer
            .slice(&found)?
            .chars()
            .take(crate::search::LIMIT + 1)
            .collect();
        Pattern::new(&query)?;
        self.last_search = Some(Confirmed {
            query,
            direction: if search == Search::WordForward {
                Direction::Forward
            } else {
                Direction::Backward
            },
            whole_word: true,
            view: self.windows.selected_id(),
            stamp: buffer.stamp(),
            found: Some(found),
        });
        Ok(())
    }
}

impl Confirmed {
    fn anchor(&self, point: Scalar, direction: Direction) -> Scalar {
        if let Some(found) = &self.found
            && (found.range.start..=found.range.end).contains(&point.0)
        {
            return Scalar(if direction == Direction::Forward {
                found.range.end
            } else {
                found.range.start
            });
        }
        point
    }
}

fn reverse(direction: Direction) -> Direction {
    if direction == Direction::Forward {
        Direction::Backward
    } else {
        Direction::Forward
    }
}
