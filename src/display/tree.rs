use crate::coordinates::Glyphs;
use crate::tree::Tree;

use super::{Band, Error, Frame, Position, Size, clip};

impl Frame {
    pub(crate) fn with_tree(mut self, tree: &Tree, size: Size, echo: &str) -> Result<Self, Error> {
        let width = tree.width(size);
        if width == 0 {
            return Ok(self);
        }
        self.size = size;
        for row in &mut self.rows {
            if row.band == Band::Echo {
                row.text = clip::line(Glyphs::for_text(echo)?, 0, size.width)?;
            } else {
                prepend(row, tree, width, size.height - 1)?;
            }
        }
        self.cursor = if tree.focused() {
            Some(Position {
                column: 0,
                row: tree.selected_row(),
            })
        } else {
            self.cursor.map(|cursor| Position {
                column: cursor.column + width + 1,
                ..cursor
            })
        };
        Ok(self)
    }
}

fn prepend(row: &mut super::Row, tree: &Tree, width: u16, height: u16) -> Result<(), Error> {
    let label = tree.line(row.row, height, width);
    let text = clip::line(Glyphs::for_text(&label)?, 0, width)?;
    row.text = format!("{text}|{}", row.text);
    row.band = Band::Mixed;
    if tree.focused() {
        row.selection = (row.row == tree.selected_row()).then_some(0..width);
        row.search = None;
    } else {
        row.selection = row
            .selection
            .take()
            .map(|range| range.start + width + 1..range.end + width + 1);
        row.search = row
            .search
            .take()
            .map(|range| range.start + width + 1..range.end + width + 1);
    }
    Ok(())
}
