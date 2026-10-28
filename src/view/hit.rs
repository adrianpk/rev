use crate::buffer::{Buffer, edit::Point};
use crate::coordinates::{Cell, Coordinates};
use crate::display::Position;
use crate::window::Area;

use super::{Error, View};

impl View {
    /// Resolve text through the same number/text partition used by painting.
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
        Ok(Coordinates::new(buffer)?
            .hit_cell(Cell { row, column }, self.left..right)?
            .map(|position| Point {
                stamp: buffer.stamp(),
                position,
            }))
    }
}
