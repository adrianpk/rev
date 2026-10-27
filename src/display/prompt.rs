use crate::coordinates::{Error as CoordinateError, Glyphs};
use crate::input::Prompt;

use super::{Error, Frame, Position, clip};

impl Frame {
    /// Preserve explicit severity supplied by the session or another presenter.
    pub fn message_kind(&mut self, message: super::Message) {
        if let Some(row) = self.rows.last_mut() {
            row.tone = super::Tone::Message(message);
        }
    }

    /// Confirmation input owns the echo cursor, leaving document point alone.
    pub(crate) fn decision(&mut self) {
        self.cursor = None;
        if self.size.width > 0 && self.size.height > 0 {
            self.cursor = Some(Position {
                column: 0,
                row: self.size.height - 1,
            });
        }
    }

    pub(crate) fn prompt(&mut self, prompt: &Prompt, notice: &str) -> Result<(), Error> {
        let Some(row) = self.rows.last_mut() else {
            return Ok(());
        };
        let prefix = format!("{}{}", prompt.label(), &prompt.text()[..prompt.cursor()]);
        let column = columns(&prefix)?;
        let left = column.saturating_sub(usize::from(self.size.width) - 1);
        let text = format!("{}{}  {notice}", prompt.label(), prompt.text());
        row.text = clip::line(Glyphs::for_text(&text)?, left, self.size.width)?;
        self.cursor = Some(Position {
            column: u16::try_from(column - left).map_err(|_| CoordinateError::Overflow)?,
            row: self.size.height - 1,
        });
        Ok(())
    }
}

fn columns(text: &str) -> Result<usize, CoordinateError> {
    let mut column = 0;
    for glyph in Glyphs::for_text(text)? {
        column = glyph?.columns.end;
    }
    Ok(column)
}
