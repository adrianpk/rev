use crate::{
    buffer::{Buffer, Span},
    command::navigation::Search,
    coordinates::{Coordinates, Scalar},
    search::{Direction, Pattern, Request},
};

/// One bounded confirmed query, qualified by the immutable copy buffer.
#[derive(Clone)]
pub(super) struct Query {
    pub text: String,
    pub direction: Direction,
    pub whole_word: bool,
    pub found: Option<Span>,
}
impl Query {
    pub fn word(buffer: &Buffer, point: Scalar, search: Search) -> Result<Self, String> {
        let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let range = coordinates
            .word_at(point)
            .map_err(|e| e.to_string())?
            .ok_or("No terminal word at point")?;
        let found = buffer
            .span(range.start.0..range.end.0)
            .map_err(|e| e.to_string())?;
        let text: String = buffer
            .slice(&found)
            .map_err(|e| e.to_string())?
            .chars()
            .take(crate::search::LIMIT + 1)
            .collect();
        Pattern::new(&text).map_err(|e| e.to_string())?;
        Ok(Self {
            text,
            direction: if search == Search::WordForward {
                Direction::Forward
            } else {
                Direction::Backward
            },
            whole_word: true,
            found: Some(found),
        })
    }
    pub fn find(
        &self,
        buffer: &Buffer,
        point: Scalar,
        search: Search,
        count: usize,
    ) -> Result<(Option<Span>, Direction), String> {
        let direction = if search == Search::Reverse {
            reverse(self.direction)
        } else {
            self.direction
        };
        let request = Request {
            anchor: self.anchor(point, direction),
            direction,
            count,
            whole_word: self.whole_word,
        };
        let found = Pattern::new(&self.text)
            .map_err(|e| e.to_string())?
            .occurrence(buffer, request)
            .map_err(|e| e.to_string())?;
        Ok((found, direction))
    }
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

pub(super) fn point(buffer: &Buffer, span: &Span, direction: Direction) -> Result<Scalar, String> {
    let point = Scalar(if direction == Direction::Forward {
        span.range.end
    } else {
        span.range.start
    });
    let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
    if coordinates.is_boundary(point).map_err(|e| e.to_string())? {
        return Ok(point);
    }
    if direction == Direction::Forward {
        coordinates.next_observed(point).map_err(|e| e.to_string())
    } else {
        coordinates
            .previous_observed(point)
            .map_err(|e| e.to_string())
    }
}

fn reverse(direction: Direction) -> Direction {
    if direction == Direction::Forward {
        Direction::Backward
    } else {
        Direction::Forward
    }
}
