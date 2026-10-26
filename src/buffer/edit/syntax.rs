use ropey::Rope;
use tree_sitter::{InputEdit, Point};

use super::{Change, Error, Prepared};

impl Prepared {
    /// Descending BASE-coordinate edits, in the same order as rope publication.
    /// The history record may group typing, but these are this exact mutation.
    pub fn syntax_edits(&self) -> &[InputEdit] {
        &self.syntax
    }
}

pub(super) fn edits(text: &Rope, changes: &[Change]) -> Result<Vec<InputEdit>, Error> {
    changes
        .iter()
        .rev()
        .map(|change| edit(text, change))
        .collect()
}

fn edit(text: &Rope, change: &Change) -> Result<InputEdit, Error> {
    let start_byte = text.char_to_byte(change.before.start);
    let start_position = position(text, change.before.start);
    Ok(InputEdit {
        start_byte,
        old_end_byte: text.char_to_byte(change.before.end),
        new_end_byte: start_byte
            .checked_add(change.inserted.len_bytes())
            .ok_or(Error::Capacity)?,
        start_position,
        old_end_position: position(text, change.before.end),
        new_end_position: advance(start_position, &change.inserted)?,
    })
}

fn position(text: &Rope, scalar: usize) -> Point {
    let row = text.char_to_line(scalar);
    Point {
        row,
        column: text.char_to_byte(scalar) - text.line_to_byte(row),
    }
}

fn advance(start: Point, inserted: &Rope) -> Result<Point, Error> {
    let rows = inserted.len_lines() - 1;
    let column = if rows == 0 {
        start
            .column
            .checked_add(inserted.len_bytes())
            .ok_or(Error::Capacity)?
    } else {
        inserted.len_bytes() - inserted.line_to_byte(rows)
    };
    Ok(Point {
        row: start.row.checked_add(rows).ok_or(Error::Capacity)?,
        column,
    })
}
