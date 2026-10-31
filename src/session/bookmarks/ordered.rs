use super::*;
use crate::bookmark::RecordId;

pub(in crate::session) struct Iteration {
    view: ViewId,
    point: Point,
    generation: u64,
    id: RecordId,
}
impl Session {
    pub(in crate::session) fn remember_bookmark_iteration(&mut self, id: RecordId) {
        let generation = self.bookmarks().generation();
        self.bookmark_iteration = Some(Iteration {
            view: self.windows.selected_id(),
            point: self.view().point,
            generation,
            id,
        });
    }
    pub(super) fn step_bookmark(&mut self, backward: bool) -> Action {
        let load_error = self.ensure_bookmarks().err();
        let point = self.view().point;
        let buffer = match self.buffer() {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        let path = match Bookmarks::target(buffer, point) {
            Ok(path) => path,
            Err(error) => return self.error(&error),
        };
        let records = self.bookmarks();
        let generation = records.generation();
        let locations = records.ordered(buffer.stamp(), &path);
        if locations.is_empty() {
            let retained = records.iter().filter(|r| r.path() == path).count();
            drop(locations);
            drop(records);
            return self.message(&load_error.unwrap_or_else(||format!("No resolved bookmarks in the current file; {retained} retained unresolved or stale targets; jump by name to check them")));
        }
        let repeated = self
            .bookmark_iteration
            .as_ref()
            .filter(|pin| {
                pin.view == self.windows.selected_id()
                    && pin.point == point
                    && pin.generation == generation
            })
            .and_then(|pin| locations.iter().position(|r| r.id == pin.id));
        let index = if let Some(index) = repeated {
            if backward {
                (index + locations.len() - 1) % locations.len()
            } else {
                (index + 1) % locations.len()
            }
        } else if backward {
            locations
                .iter()
                .rposition(|r| r.position <= point.position.0)
                .unwrap_or(locations.len() - 1)
        } else {
            locations
                .iter()
                .position(|r| r.position >= point.position.0)
                .unwrap_or(0)
        };
        let name = locations[index].name.to_owned();
        drop(locations);
        drop(records);
        match self.visit_bookmark_from(&name, navigation::Origin::Ordered(generation)) {
            Ok(()) => Action::Redraw,
            Err(error) => self.error(&error),
        }
    }
}
