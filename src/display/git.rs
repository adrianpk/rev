use super::*;
use crate::coordinates::Error as CoordinateError;
use std::os::unix::ffi::OsStrExt;

/// OS debug escaping preserves distinguishable labels for invalid UTF-8 and
/// controls. These labels are never reparsed into repository arguments.
pub(crate) fn label(bytes: &[u8]) -> String {
    let text = format!("{:?}", std::ffi::OsStr::from_bytes(bytes));
    text.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(&text)
        .to_owned()
}

impl Frame {
    pub(crate) fn git_popup(
        &mut self,
        composition: &crate::panel::Composition,
    ) -> Result<(), Error> {
        for border in composition.borders() {
            self.projection_text("", *border)?;
        }
        Ok(())
    }

    pub(crate) fn git(
        buffer: &Buffer,
        size: Size,
        palette: &crate::config::Palette,
        echo: &str,
        admission: crate::git::budget::Lease,
        mut line: impl FnMut(usize) -> (String, bool),
    ) -> Result<Self, Error> {
        let mut frame = Self::pending(buffer, size, palette)?;
        frame._git_admission = Some(admission);
        if size.width == 0 || size.height == 0 {
            return Ok(frame);
        }
        frame.rows.pop();
        frame.echo(echo)?;
        for row in 0..size.height.saturating_sub(2) {
            let (text, selected) = line(usize::from(row));
            let area = Area::new(0, row, size.width, 1).ok_or(CoordinateError::Overflow)?;
            frame.projection_text(&text, area)?;
            if selected {
                frame.rows[usize::from(row)].tone = Tone::GitSelection;
                frame.rows[usize::from(row)].faces.clear();
            }
        }
        if size.height >= 2 {
            let row = size.height - 2;
            frame.blit(
                Area::new(0, row, size.width, 1).ok_or(CoordinateError::Overflow)?,
                Row {
                    row: 0,
                    text: clip::line(
                        Glyphs::for_text(
                            " Git Status  g refresh  Tab section  ? actions  q return",
                        )?,
                        0,
                        size.width,
                    )?,
                    band: Band::Status,
                    tone: Tone::Status(true),
                    faces: Vec::new(),
                    syntax: Vec::new(),
                    selection: None,
                    search: None,
                },
            )?;
        }
        frame.cursor = None;
        Ok(frame)
    }
}
