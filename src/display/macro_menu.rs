use super::*;
impl Frame {
    /// Read-only component text is projected only into its owned visible pane.
    pub(crate) fn macro_menu(&mut self, lines: &[String], panel: Panel) -> Result<(), Error> {
        let area = panel.content();
        for offset in 0..area.height() {
            let line = Area::new(area.column(), area.row() + offset, area.width(), 1)
                .ok_or(crate::coordinates::Error::Overflow)?;
            self.projection_text(lines.get(offset as usize).map_or("", String::as_str), line)?;
        }
        if let Some(status) = panel.status() {
            self.projection_text("*Keyboard Macros*  ? help  g refresh  q return", status)?;
        }
        if panel.active() {
            self.cursor = None;
        }
        Ok(())
    }
}
