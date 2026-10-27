use crate::coordinates::{Error as CoordinateError, Glyphs};
use crate::input::Prompt;
use crate::window::Area;

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
        let Some(area) = Area::new(0, self.size.height.saturating_sub(1), self.size.width, 1)
        else {
            return Ok(());
        };
        self.prompt_in(prompt, notice, area)
    }

    pub(super) fn prompt_in(
        &mut self,
        prompt: &Prompt,
        notice: &str,
        area: Area,
    ) -> Result<(), Error> {
        if area.width() == 0 || self.rows.is_empty() {
            return Ok(());
        }
        let tone = self
            .rows
            .last()
            .map_or(super::Tone::Document, |row| row.tone);
        let prefix = format!("{}{}", prompt.label(), &prompt.text()[..prompt.cursor()]);
        let column = columns(&prefix)?;
        let left = column.saturating_sub(usize::from(area.width()) - 1);
        let text = format!("{}{}  {notice}", prompt.label(), prompt.text());
        self.blit(
            area,
            super::Row {
                row: 0,
                text: clip::line(Glyphs::for_text(&text)?, left, area.width())?,
                band: super::Band::Echo,
                tone,
                faces: Vec::new(),
                syntax: Vec::new(),
                selection: None,
                search: None,
            },
        )?;
        self.cursor = Some(Position {
            column: area.column()
                + u16::try_from(column - left).map_err(|_| CoordinateError::Overflow)?,
            row: area.row(),
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
