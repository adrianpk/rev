use super::{Cell, Coordinates, Error, Scalar};

impl Coordinates<'_> {
    /// Logical grapheme columns count a source tab or wide cluster once.
    /// Display, hit testing and wrapping continue to use terminal-cell columns.
    pub(crate) fn grapheme_cell(&self, point: Scalar) -> Result<Cell, Error> {
        if !self.is_boundary(point)? {
            return Err(Error::Cluster(point));
        }
        let row = self.to_line(point)?.row;
        let mut column = 0;
        for glyph in self.glyphs(row)? {
            if glyph?.source.start >= point {
                break;
            }
            column += 1;
        }
        Ok(Cell { row, column })
    }

    pub(crate) fn from_grapheme(&self, cell: Cell) -> Result<Scalar, Error> {
        for (column, glyph) in self.glyphs(cell.row)?.enumerate() {
            if column == cell.column {
                return Ok(glyph?.source.start);
            }
            glyph?;
        }
        let start = self.from_cell(Cell {
            row: cell.row,
            column: 0,
        })?;
        self.line_end(start)
    }
}
