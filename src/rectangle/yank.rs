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
}

impl Rectangle {
    pub(crate) fn insertions(&self, buffer: &Buffer, point: Point) -> Result<Insertion, Error> {
        buffer.slice(&Span {
            stamp: point.stamp,
            range: point.position.0..point.position.0,
        })?;
        let start = Coordinates::new(buffer)?.to_cell(point.position)?;
        let mut insertion = Insertion {
            edits: Vec::new(),
            cursor: point.position,
            inserted: 0,
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
        let position = coordinates.from_cell(cell)?;
        let prefix = if text.is_empty() {
            0
        } else {
            padding(&coordinates, position, cell.column)?
        };
        let mut content = " ".repeat(prefix);
        content.push_str(text);
        self.append(buffer, position, content)
    }

    fn extend(&mut self, buffer: &Buffer, column: usize, text: &str) -> Result<(), Error> {
        let mut content = buffer.newline().to_owned();
        if !text.is_empty() {
            content.extend(std::iter::repeat_n(' ', column));
            content.push_str(text);
        }
        self.append(buffer, Scalar(buffer.len_chars()), content)
    }

    fn append(&mut self, buffer: &Buffer, position: Scalar, text: String) -> Result<(), Error> {
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
        self.edits.push(Replacement {
            source: buffer.span(position.0..position.0)?,
            text,
        });
        Ok(())
    }
}

fn padding(coordinates: &Coordinates<'_>, position: Scalar, column: usize) -> Result<usize, Error> {
    // Cell lookup keeps whole graphemes. A short row or an interior glyph
    // therefore needs a left gap before the retained text, never right padding.
    Ok(column.saturating_sub(coordinates.to_cell(position)?.column))
}
