use std::ops::Range;

use super::{Cell, Coordinates, Error, Scalar};

impl Coordinates<'_> {
    /// Only complete painted glyphs and the exact EOL boundary represent text.
    /// Clipped fragments and trailing fill are spaces owned by the viewport.
    pub(crate) fn hit_cell(
        &self,
        cell: Cell,
        visible: Range<usize>,
    ) -> Result<Option<Scalar>, Error> {
        let checkpoint = self.column_checkpoint(cell);
        let mut end = checkpoint.map_or(Scalar(0), |(point, _)| point);
        let mut column = checkpoint.map_or(0, |(_, column)| column);
        for glyph in self.glyphs_from_column(cell)? {
            let glyph = glyph?;
            if cell.column < glyph.columns.end {
                return Ok((glyph.columns.start >= visible.start
                    && glyph.columns.end <= visible.end)
                    .then_some(glyph.source.start));
            }
            end = glyph.source.end;
            column = glyph.columns.end;
        }
        if cell.column != column {
            return Ok(None);
        }
        // Empty rows have no glyph from which to obtain their canonical start.
        if column == 0 {
            end = self.from_cell(cell)?;
        }
        Ok(Some(end))
    }
}
