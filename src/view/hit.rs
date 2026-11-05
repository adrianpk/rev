use crate::buffer::{Buffer, edit::Point};
use crate::coordinates::{Cell, Coordinates};
use crate::display::Position;
use crate::window::Area;

use super::{Error, Fragment, View, indentation::Margins};

impl View {
    /// Resolve text through the same gutter/text partition used by painting.
    /// A source row beyond EOF or a non-text cell cannot invent a document point.
    pub fn hit(
        &self,
        buffer: &Buffer,
        content: Area,
        position: Position,
    ) -> Result<Option<Point>, Error> {
        self.cell(buffer)?;
        let text = self.gutter_geometry(buffer, content)?.text();
        let Some(local) = text.local(position) else {
            return Ok(None);
        };
        self.text_hit(buffer, text, local)
    }

    fn text_hit(
        &self,
        buffer: &Buffer,
        text: Area,
        local: Position,
    ) -> Result<Option<Point>, Error> {
        if buffer.soft_wrap() {
            return self.wrapped_hit(buffer, text, local);
        }
        // Arithmetic overflow is an invalid viewport, distinct from an ordinary
        // blank row beyond EOF. Preserve the native checked-error contract.
        let projected = self
            .projected_top(buffer)
            .checked_add(usize::from(local.row))
            .ok_or(crate::coordinates::Error::Overflow)?;
        let Some(row) = self.source_at(buffer, projected) else {
            return Ok(None);
        };
        let column = self
            .left
            .checked_add(usize::from(local.column))
            .ok_or(crate::coordinates::Error::Overflow)?;
        let right = self
            .left
            .checked_add(usize::from(text.width()))
            .ok_or(crate::coordinates::Error::Overflow)?;
        let margin = Margins::new(buffer, row, text.width()).at(0);
        if column < margin {
            return Ok(Some(Point {
                stamp: buffer.stamp(),
                position: Coordinates::new(buffer)?.from_cell(Cell { row, column: 0 })?,
            }));
        }
        Ok(Coordinates::new(buffer)?
            .hit_cell(
                Cell {
                    row,
                    column: column - margin,
                },
                self.left.saturating_sub(margin)..right.saturating_sub(margin),
            )?
            .map(|position| Point {
                stamp: buffer.stamp(),
                position,
            }))
    }
    fn wrapped_hit(
        &self,
        buffer: &Buffer,
        text: Area,
        local: Position,
    ) -> Result<Option<Point>, Error> {
        let fragments = self.wrapped_rows(buffer, text.width(), local.row + 1)?;
        let Some(fragment) = fragments.get(usize::from(local.row)) else {
            return Ok(None);
        };
        Ok(fragment
            .hit(buffer, text.width(), local.column)?
            .map(|position| Point {
                stamp: buffer.stamp(),
                position,
            }))
    }
}

impl Fragment {
    /// Padding selects the real fragment start. Partly clipped glyphs and
    /// trailing fill retain the coordinate owner's existing rejection policy.
    pub(crate) fn hit(
        &self,
        buffer: &Buffer,
        width: u16,
        column: u16,
    ) -> Result<Option<crate::coordinates::Scalar>, Error> {
        self.check_projection(buffer)?;
        if self.layout_width() != width {
            return Err(crate::coordinates::Error::Pending.into());
        }
        if column >= width {
            return Ok(None);
        }
        if column < self.margin {
            return Ok(Some(self.source.start));
        }
        let right = self
            .columns
            .start
            .checked_add(usize::from(width.saturating_sub(self.margin)))
            .ok_or(crate::coordinates::Error::Overflow)?;
        Ok(Coordinates::new(buffer)?
            .hit_cell(
                Cell {
                    row: self.line,
                    column: self.source_column(usize::from(column)),
                },
                self.columns.start..right,
            )?
            .filter(|point| self.contains(*point, width)))
    }
}
