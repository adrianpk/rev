use crate::buffer::{
    Buffer, Span,
    edit::{Error, Point, Replacement},
};
use crate::coordinates::{Cell, Coordinates, Scalar};

use super::Rectangle;

/// Ordered insertions in one BASE revision. All new lines share one EOF span,
/// merged with an existing last-row insertion when their base offsets coincide.
pub(crate) struct Insertion {
    pub edits: Vec<Replacement>,
    pub cursor: Scalar,
    inserted: usize,
    bytes: usize,
    columns: super::Columns,
}

impl Rectangle {
    pub(crate) fn insertions(&self, buffer: &Buffer, point: Point) -> Result<Insertion, Error> {
        buffer.slice(&Span {
            stamp: point.stamp,
            range: point.position.0..point.position.0,
        })?;
        let start = self
            .columns
            .cell(&Coordinates::new(buffer)?, point.position)?;
        let mut insertion = Insertion {
            edits: Vec::new(),
            cursor: point.position,
            inserted: 0,
            bytes: 0,
            columns: self.columns,
        };
        for (index, text) in self.rows.iter().enumerate() {
            let row = start.row.checked_add(index).ok_or(Error::Capacity)?;
            let cell = Cell {
                row,
                column: start.column,
            };
            if row < buffer.len_lines() {
                insertion.row(buffer, cell, text)?;
            } else {
                insertion.extend(buffer, start.column, text)?;
            }
        }
        Ok(insertion)
    }
}

impl Insertion {
    fn row(&mut self, buffer: &Buffer, cell: Cell, text: &str) -> Result<(), Error> {
        let coordinates = Coordinates::new(buffer)?;
        let position = self.columns.position(&coordinates, cell)?;
        let prefix = if text.is_empty() {
            0
        } else {
            cell.column
                .saturating_sub(self.columns.cell(&coordinates, position)?.column)
        };
        buffer.check_edit_payload(prefix.saturating_add(text.len()))?;
        let mut content = " ".repeat(prefix);
        content.push_str(text);
        self.append(buffer, position, content)
    }

    fn extend(&mut self, buffer: &Buffer, column: usize, text: &str) -> Result<(), Error> {
        let payload = buffer.newline().len().saturating_add(if text.is_empty() {
            0
        } else {
            column.saturating_add(text.len())
        });
        buffer.check_edit_payload(payload)?;
        let mut content = buffer.newline().to_owned();
        if !text.is_empty() {
            content.extend(std::iter::repeat_n(' ', column));
            content.push_str(text);
        }
        self.append(buffer, Scalar(buffer.len_chars()), content)
    }

    fn append(&mut self, buffer: &Buffer, position: Scalar, text: String) -> Result<(), Error> {
        self.bytes = self.bytes.checked_add(text.len()).ok_or(Error::Capacity)?;
        buffer.check_edit_payload(self.bytes)?;
        self.inserted = self
            .inserted
            .checked_add(text.chars().count())
            .ok_or(Error::Capacity)?;
        self.cursor = Scalar(
            position
                .0
                .checked_add(self.inserted)
                .ok_or(Error::Capacity)?,
        );
        if text.is_empty() {
            return Ok(());
        }
        if let Some(previous) = self
            .edits
            .last_mut()
            .filter(|edit| edit.source.range.start == position.0)
        {
            previous.text.push_str(&text);
            return Ok(());
        }
        if buffer.edit_payload_limit().is_some() && self.edits.len() >= 64 {
            return Err(Error::Capacity);
        }
        self.edits.push(Replacement {
            source: buffer.span(position.0..position.0)?,
            text,
        });
        Ok(())
    }
}
