use crate::buffer::{
    Buffer, Span,
    edit::{Error, Replacement},
};
use crate::coordinates::{Cell, Coordinates, Scalar};

use super::{Part, Selection};

impl Selection {
    /// All row replacements refer to the same immutable BASE revision. Empty
    /// source rows contribute no edit, and line separators remain outside spans.
    pub(crate) fn deletions(&self, buffer: &Buffer) -> Result<Vec<Replacement>, Error> {
        let mut edits = Vec::new();
        for row in self.rows.clone() {
            if let Some(edit) = self.deletion(buffer, row)? {
                edits.push(edit);
            }
        }
        Ok(edits)
    }

    fn deletion(&self, buffer: &Buffer, row: usize) -> Result<Option<Replacement>, Error> {
        let mut parts = self.parts(buffer, row, None)?;
        let Some(first) = parts.next().transpose()? else {
            return Ok(None);
        };
        let mut removal = Removal {
            source: first.source,
            padding: first.padding,
        };
        for part in parts {
            removal.include(part?)?;
        }
        Ok(Some(Replacement {
            source: removal.source,
            text: " ".repeat(removal.padding),
        }))
    }

    /// No edit precedes the top-left point. A partially touched leading tab
    /// leaves its unselected prefix spaces before the final initiating cursor.
    pub(crate) fn cursor(&self, buffer: &Buffer) -> Result<Scalar, Error> {
        let row = *self.rows.start();
        let position = Coordinates::new(buffer)?.from_cell(Cell {
            row,
            column: self.columns.start,
        })?;
        let prefix = self
            .parts(buffer, row, None)?
            .next()
            .transpose()?
            .map_or(0, |part| part.prefix);
        Ok(Scalar(
            position.0.checked_add(prefix).ok_or(Error::Capacity)?,
        ))
    }
}

struct Removal {
    source: Span,
    padding: usize,
}

impl Removal {
    fn include(&mut self, part: Part) -> Result<(), Error> {
        self.source.range.end = part.source.range.end;
        self.padding = self
            .padding
            .checked_add(part.padding)
            .ok_or(Error::Capacity)?;
        Ok(())
    }
}
