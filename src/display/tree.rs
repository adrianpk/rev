use crate::buffer::Buffers;
use crate::coordinates::Glyphs;
use crate::panel::{Composition, Panel};
use crate::tree::Tree;
use crate::window::Windows;

use super::{Band, Error, Frame, Position, Row, clip};

impl Frame {
    /// Render one supplied workspace composition without shifting content after
    /// projection. Semantic tree focus remains with the supplied Tree owner.
    pub fn compose_workspace(
        buffers: &Buffers,
        windows: &Windows,
        tree: &Tree,
        composition: &Composition,
        echo: &str,
    ) -> Result<Self, Error> {
        let mut frame = Self::compose_in(buffers, windows, composition, echo)?;
        frame.tree(tree, composition.tree())?;
        Ok(frame)
    }

    pub(crate) fn tree(&mut self, tree: &Tree, panel: Option<Panel>) -> Result<(), Error> {
        let Some(panel) = panel else {
            return Ok(());
        };
        let content = panel.content();
        for row in 0..content.height() {
            let label = tree.line(row)?;
            let selected = tree.row_selected(row);
            // The tree and a live source region can occupy the same screen row.
            // Its selected face preserves both independent surface indications.
            let source_selection = self.rows[usize::from(content.row() + row)]
                .selection
                .is_some();
            let source = Row {
                row,
                text: clip::line(
                    Glyphs::for_text(&label)?,
                    usize::from(tree.horizontal_offset()),
                    content.width(),
                )?,
                syntax: Vec::new(),
                band: Band::Document,
                tone: if selected {
                    super::Tone::TreeSelected
                } else {
                    super::Tone::Tree(tree.row_kind(row))
                },
                faces: Vec::new(),
                search: None,
                selection: (selected && !source_selection).then_some(0..content.width()),
            }
            .projected(panel);
            self.blit(content, source)?;
        }
        self.tree_status(tree, panel)?;
        if panel.active() {
            self.cursor = tree
                .selected_row()
                .and_then(|row| panel.position(Position { column: 0, row }));
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
                    tone: super::Tone::Status(panel.active()),
                    faces: Vec::new(),
                    search: None,
                    selection: None,
                },
            )?;
        }
        Ok(())
    }
}
