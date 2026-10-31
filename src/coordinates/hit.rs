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
        let glyphs = self.glyphs_from_column(cell)?;
        let checkpoint = self.column_checkpoint(cell);
        let mut end = checkpoint.map_or(Scalar(0), |(point, _)| point);
        let mut column = checkpoint.map_or(0, |(_, column)| column);
        for glyph in glyphs {
            let glyph = glyph?;
            if cell.column < glyph.columns.end {
                self.remember(
                    glyph.source.start,
                    Cell {
                        row: cell.row,
                        column: glyph.columns.start,
                    },
                );
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
        self.remember(
            end,
            Cell {
                row: cell.row,
                column,
            },
        );
        Ok(Some(end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffers;

    // Warming a cold horizontal query can start its iterator exactly at EOL.
    // That empty suffix must retain its real column and reject later padding.
    #[test]
    fn cold_eol() {
        for (source, column, expected) in [
            ("abc", 3, Some(3)),
            ("abc", 4, None),
            ("", 0, Some(0)),
            ("界", 2, Some(1)),
        ] {
            let mut buffers = Buffers::new().unwrap();
            let id = buffers.create(source).unwrap();
            let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
            assert_eq!(
                coordinates
                    .hit_cell(Cell { row: 0, column }, 0..10)
                    .unwrap(),
                expected.map(Scalar)
            );
        }
    }
}
