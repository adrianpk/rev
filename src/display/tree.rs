use crate::coordinates::Glyphs;
use crate::panel::Panel;
use crate::tree::Tree;

use super::{Band, Error, Frame, Position, Row, clip};

impl Frame {
    pub(crate) fn tree(&mut self, tree: &Tree, panel: Option<Panel>) -> Result<(), Error> {
        let Some(panel) = panel else {
            return Ok(());
        };
        let content = panel.content();
        for row in 0..content.height() {
            let label = tree.line(row, content.width());
            let source = Row {
                row,
                text: clip::line(Glyphs::for_text(&label)?, 0, content.width())?,
                syntax: Vec::new(),
                band: Band::Document,
                search: None,
                selection: (panel.active() && row == tree.selected_row())
                    .then_some(0..content.width()),
            }
            .projected(panel);
            self.blit(content, source)?;
        }
        self.tree_status(tree, panel)?;
        if panel.active() {
            self.cursor = panel.position(Position {
                column: 0,
                row: tree.selected_row(),
            });
        }
        Ok(())
    }
    fn tree_status(&mut self, tree: &Tree, panel: Panel) -> Result<(), Error> {
        if let Some(status) = panel.status() {
            self.blit(
                status,
                Row {
                    row: 0,
                    text: clip::line(Glyphs::for_text(tree.status())?, 0, status.width())?,
                    syntax: Vec::new(),
                    band: Band::Status,
                    search: None,
                    selection: None,
                },
            )?;
        }
        Ok(())
    }
}
