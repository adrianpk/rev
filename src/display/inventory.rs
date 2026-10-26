use crate::buffer::{Buffer, BufferId, Buffers};
use crate::coordinates::Glyphs;

use super::{Error, Frame, Position, clip};

impl Frame {
    /// Derive only visible inventory rows; the list never owns editable text.
    pub(crate) fn inventory(&mut self, buffers: &Buffers, selected: BufferId) -> Result<(), Error> {
        let extent = usize::from(self.size.text_rows());
        if extent == 0 {
            self.decision();
            return Ok(());
        }
        let index = buffers
            .iter()
            .position(|buffer| buffer.id() == selected)
            .ok_or(crate::buffer::Error::Missing(selected))?;
        let top = index.saturating_sub(extent - 1);
        for row in self.rows.iter_mut().take(extent) {
            row.text = " ".repeat(usize::from(self.size.width));
        }
        for (row, buffer) in self
            .rows
            .iter_mut()
            .take(extent)
            .zip(buffers.iter().skip(top))
        {
            row.text = clip::line(
                Glyphs::for_text(&entry(buffer, selected))?,
                0,
                self.size.width,
            )?;
        }
        self.rows[extent].text = clip::line(
            Glyphs::for_text(&format!("Buffers: {} live", buffers.iter().len()))?,
            0,
            self.size.width,
        )?;
        self.cursor = Some(Position {
            column: 0,
            row: u16::try_from(index - top).map_err(|_| crate::coordinates::Error::Overflow)?,
        });
        Ok(())
    }
}

fn entry(buffer: &Buffer, selected: BufferId) -> String {
    let marker = if buffer.id() == selected { ">" } else { " " };
    let dirty = if buffer.is_modified() { "**" } else { "--" };
    let access = if buffer.is_read_only() { "RO" } else { "RW" };
    let path = buffer
        .file()
        .map_or_else(|| "(unnamed)".into(), |file| format!("{:?}", file.path()));
    format!("{marker} {} {dirty} {access} {path}", buffer.name())
}
