use super::*;
use crate::window::Area;
/// Add a disposable timer status only where a complete active-pane label fits.
/// The face range protects adjacent panes and the normal narrow status tokens.
impl Frame {
    pub(crate) fn timer_status(&mut self, label: &str) -> Result<(), Error> {
        let cells = crate::coordinates::Glyphs::for_text(label)?
            .try_fold(0, |_, glyph| glyph.map(|g| g.columns.end))?;
        let mut target = None;
        for row in &self.rows {
            if let Some(face) = row
                .faces
                .iter()
                .find(|face| face.tone == Tone::Status(true))
            {
                let width = face.columns.end - face.columns.start;
                if usize::from(width) >= cells + 12 {
                    let column = face.columns.end - cells as u16;
                    target = Area::new(column, row.row, cells as u16, 1);
                }
                break;
            }
        }
        if let Some(area) = target {
            self.blit(
                area,
                Row {
                    row: 0,
                    text: label.into(),
                    band: Band::Status,
                    tone: Tone::Status(true),
                    faces: Vec::new(),
                    syntax: Vec::new(),
                    selection: None,
                    search: None,
                },
            )?;
        }
        Ok(())
    }
}
