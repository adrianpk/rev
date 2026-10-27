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
        let row = self
            .top
            .checked_add(usize::from(local.row))
            .ok_or(crate::coordinates::Error::Overflow)?;
        if row >= buffer.len_lines() {
            return Ok(None);
        }
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
