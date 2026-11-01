use crate::buffer::Stamp;
use crate::command::{
    Navigation,
    navigation::{Find, Target},
};
use crate::coordinates::{Coordinates, Found, Scalar};
use crate::view::Error;
use crate::window::ViewId;

use super::super::{Action, Session};

pub(in crate::session) struct State {
    view: ViewId,
    stamp: Stamp,
    find: Find,
    character: char,
    found: Found,
}

impl Session {
    pub(super) fn navigation_argument(&mut self, request: Navigation) -> Option<Action> {
        if matches!(request.target, Target::Find(_)) && request.character.is_none() {
            let outcome = self.input.expect_navigation(request);
            return Some(self.outcome(outcome, std::time::Instant::now()));
        }
        if matches!(request.target, Target::RepeatFind(_)) && !self.find_valid() {
            return Some(self.message("No current find to repeat"));
        }
        None
    }

    fn find_valid(&self) -> bool {
        self.last_find.as_ref().is_some_and(|state| {
            state.view == self.windows.selected_id() && state.stamp == self.view().point.stamp
        })
    }

    pub(super) fn navigation_find(&self, request: Navigation) -> Result<Option<Found>, Error> {
        let Some((find, character, origin)) = self.find_descriptor(request) else {
            return Ok(None);
        };
        Ok(Coordinates::new(self.buffer()?)?.find_target(
            origin,
            find,
            character,
            request.amount(),
        )?)
    }

    fn find_descriptor(&self, request: Navigation) -> Option<(Find, char, Scalar)> {
        let origin = self.view().point.position;
        match request.target {
            Target::Find(find) => Some((find, request.character?, origin)),
            Target::RepeatFind(reverse) if self.find_valid() => {
                let state = self.last_find.as_ref()?;
                let origin = if origin == state.found.point {
                    state.found.matched
                } else {
                    origin
                };
                Some((
                    if reverse {
                        state.find.reversed()
                    } else {
                        state.find
                    },
                    state.character,
                    origin,
                ))
            }
            _ => None,
        }
    }

    pub(super) fn retain_find(&mut self, request: Navigation, found: Option<Found>) {
        let Some(found) = found else { return };
        if let Target::Find(find) = request.target {
            self.last_find = Some(State {
                view: self.windows.selected_id(),
                stamp: self.view().point.stamp,
                find,
                character: request.character.expect("successful find had a character"),
                found,
            });
        } else if let Some(state) = &mut self.last_find {
            state.found = found;
        }
    }
}
