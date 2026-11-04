use crate::{
    buffer::{Buffer, Span},
    coordinates::{Coordinates, Scalar},
    search::{Direction, Pattern, Request},
    view::View,
};

pub(super) struct Search {
    pub origin: View,
    pub found: Option<Span>,
    direction: Direction,
    anchor: Scalar,
}
impl Search {
    pub fn new(view: &View, direction: Direction) -> Self {
        Self {
            origin: view.clone(),
            found: None,
            direction,
            anchor: view.point.position,
        }
    }
    pub fn repeat(&mut self, direction: Direction) {
        if let Some(found) = &self.found {
            self.anchor = Scalar(if direction == Direction::Forward {
                found.range.end
            } else {
                found.range.start
            });
        }
        self.direction = direction;
    }
    pub fn update(&mut self, buffer: &Buffer, query: &str) -> Result<Option<Scalar>, String> {
        if query.is_empty() {
            self.found = None;
            return Ok(Some(self.origin.point.position));
        }
        let pattern = Pattern::new(query).map_err(|e| e.to_string())?;
        self.found = pattern
            .occurrence(buffer, Request::new(self.anchor, self.direction))
            .map_err(|e| e.to_string())?;
        let Some(found) = &self.found else {
            return Ok(None);
        };
        let point = Scalar(if self.direction == Direction::Forward {
            found.range.end
        } else {
            found.range.start
        });
        let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        if coordinates.is_boundary(point).map_err(|e| e.to_string())? {
            return Ok(Some(point));
        }
        coordinates
            .next_observed(point)
            .map(Some)
            .map_err(|e| e.to_string())
    }
}
