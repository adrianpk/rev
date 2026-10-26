use std::ops::Range;

use crate::buffer::edit::{Cursor, Error, Replacement};
use crate::coordinates::{Coordinates, Scalar};

use super::Session;

impl Session {
    pub(super) fn line_break(&mut self) -> Result<bool, Error> {
        let newline = self.buffer()?.newline();
        let point = self.view.point.position.0;
        self.replace(point..point, newline)
    }

    pub(super) fn delete(&mut self, backward: bool) -> Result<bool, Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let point = self.view.point.position;
        let range = if backward {
            coordinates.previous(point)?.0..point.0
        } else {
            point.0..coordinates.next(point)?.0
        };
        self.replace(range, "")
    }

    pub(super) fn replace(&mut self, range: Range<usize>, text: &str) -> Result<bool, Error> {
        if range.is_empty() && text.is_empty() {
            return Ok(false);
        }
        let buffer = self.buffers.get_mut(self.view.point.stamp.buffer)?;
        let source = buffer.span(range.clone())?;
        let position = range
            .start
            .checked_add(text.chars().count())
            .ok_or(Error::Capacity)?;
        let edits = [Replacement {
            source,
            text: text.into(),
        }];
        let cursor = Cursor {
            index: 0,
            position: Scalar(position),
        };
        let prepared =
            buffer.prepare(&edits, std::slice::from_ref(&self.view.point), Some(cursor))?;
        buffer.publish(prepared, std::slice::from_mut(&mut self.view.point))?;
        Ok(true)
    }
}
