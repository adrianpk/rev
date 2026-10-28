use std::ops::Range;

use crate::buffer::edit::{Cursor, Error, Replacement};
use crate::config::IndentStyle;
use crate::coordinates::{Coordinates, Scalar};

use super::Session;

impl Session {
    pub(super) fn indent(&mut self) -> Result<bool, Error> {
        let buffer = self.buffer()?;
        let point = self.windows.selected().point.position;
        let policy = buffer.policy();
        let text = match policy.indent_style {
            IndentStyle::Tabs => "\t".to_owned(),
            IndentStyle::Spaces => {
                let column = Coordinates::new(buffer)?.to_cell(point)?.column;
                " ".repeat(policy.indent_width - column % policy.indent_width)
            }
        };
        self.replace(point.0..point.0, &text)
    }

    pub(super) fn line_break(&mut self) -> Result<bool, Error> {
        let newline = self.buffer()?.newline();
        let point = self.windows.selected().point.position.0;
        self.replace(point..point, newline)
    }

    pub(super) fn delete(&mut self, backward: bool) -> Result<bool, Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let point = self.windows.selected().point.position;
        let range = if backward {
            coordinates.previous(point)?.0..point.0
        } else {
            point.0..coordinates.next(point)?.0
        };
        self.replace(range, "")
    }

    /// Replace a scalar range in the selected buffer through the same checked
    /// transaction as editing commands, reconciling every live shared view.
    pub fn replace(&mut self, range: Range<usize>, text: &str) -> Result<bool, Error> {
        self.cancel_pointer_capture();
        self.yank = None;
        self.kills.break_chain();
        self.edit(range, text, false)
    }

    pub(super) fn type_text(&mut self, text: &str) -> Result<bool, Error> {
        let point = self.windows.selected().point.position.0;
        self.edit(point..point, text, true)
    }

    fn edit(&mut self, range: Range<usize>, text: &str, typing: bool) -> Result<bool, Error> {
        if range.is_empty() && text.is_empty() {
            return Ok(false);
        }
        let buffer = self
            .buffers
            .get_mut(self.windows.selected().point.stamp.buffer)?;
        let (edit, position) = replacement(buffer, range, text)?;
        let points = self.windows.points(buffer)?;
        let cursor = Cursor {
            index: points.initiator,
            position,
        };
        let mut prepared = if typing {
            buffer.prepare_typing(&edit, &points.values, cursor)?
        } else {
            buffer.prepare(std::slice::from_ref(&edit), &points.values, Some(cursor))?
        };
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        Ok(true)
    }
}

fn replacement(
    buffer: &crate::buffer::Buffer,
    range: Range<usize>,
    text: &str,
) -> Result<(Replacement, Scalar), Error> {
    let source = buffer.span(range.clone())?;
    let position = range
        .start
        .checked_add(text.chars().count())
        .ok_or(Error::Capacity)?;
    Ok((
        Replacement {
            source,
            text: text.into(),
        },
        Scalar(position),
    ))
}
