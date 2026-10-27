use crate::buffer::Buffer;
use crate::coordinates::{Cell, Error, Glyphs};

use super::clip;

/// Disposable observations from one view and its canonical buffer. Status owns
/// no text or policy and uses the original pane width, independently of gutters.
struct Facts<'a> {
    name: &'a str,
    state: &'static str,
    access: &'static str,
    mode: &'static str,
    marker: char,
}

pub(super) fn line(
    buffer: &Buffer,
    point: Cell,
    width: u16,
    active: bool,
) -> Result<String, Error> {
    let row = point.row.checked_add(1).ok_or(Error::Overflow)?;
    let column = point.column.checked_add(1).ok_or(Error::Overflow)?;
    Facts::new(buffer, active).render(width, row, column)
}

impl<'a> Facts<'a> {
    fn new(buffer: &'a Buffer, active: bool) -> Self {
        Self {
            name: buffer.name(),
            state: if buffer.is_modified() { "**" } else { "--" },
            access: if buffer.is_readonly() { "RO" } else { "RW" },
            mode: buffer.mode(),
            marker: if active { '>' } else { ' ' },
        }
    }

    fn render(&self, width: u16, row: usize, column: usize) -> Result<String, Error> {
        let position = format!("L{row} C{column}");
        let name = prefix(self.name, width)?;
        let full = self.left(&name, Some(&position));
        let used = cells(&full)?;
        let right = cells(self.mode)?;
        if used + 1 + right <= usize::from(width) {
            let gap = usize::from(width) - used - right;
            return Ok(format!("{full}{}{mode}", " ".repeat(gap), mode = self.mode));
        }
        if used <= usize::from(width) {
            return padded(&full, width);
        }
        self.narrow(width, &format!("L{row}:{column}"))
    }

    fn narrow(&self, width: u16, compact: &str) -> Result<String, Error> {
        if let Some(left) = self.clipped(width, Some(compact))? {
            return padded(&left, width);
        }
        if let Some(left) = self.clipped(width, None)? {
            return padded(&left, width);
        }
        padded(&self.tiny(width), width)
    }

    fn left(&self, name: &str, position: Option<&str>) -> String {
        let name = if name.is_empty() {
            String::new()
        } else {
            format!("{name} ")
        };
        let position = position.map_or_else(String::new, |value| format!(" -- {value}"));
        format!(
            "{} {name}{} {}{position}",
            self.marker, self.state, self.access
        )
    }

    fn clipped(&self, width: u16, position: Option<&str>) -> Result<Option<String>, Error> {
        let base = self.left("", position);
        let Some(remaining) = usize::from(width).checked_sub(cells(&base)?) else {
            return Ok(None);
        };
        let budget = u16::try_from(remaining.saturating_sub(1)).map_err(|_| Error::Overflow)?;
        let name = prefix(self.name, budget)?;
        Ok(Some(self.left(&name, position)))
    }

    // Never show half an access token or a truncated numeric position. Below
    // five cells, keep the complete state marker when possible; one cell keeps focus.
    fn tiny(&self, width: u16) -> String {
        match width {
            0 => String::new(),
            1 => self.marker.to_string(),
            2 => self.state.into(),
            3 => format!("{}{}", self.marker, self.state),
            4 => format!("{} {}", self.marker, self.state),
            5 => format!("{} {}", self.state, self.access),
            _ => format!("{}{} {}", self.marker, self.state, self.access),
        }
    }
}

fn cells(text: &str) -> Result<usize, Error> {
    Glyphs::for_text(text)?.try_fold(0, |_, glyph| glyph.map(|glyph| glyph.columns.end))
}

fn prefix(text: &str, width: u16) -> Result<String, Error> {
    let used = cells(text)?.min(usize::from(width));
    clip::line(Glyphs::for_text(text)?, 0, used as u16)
}

fn padded(text: &str, width: u16) -> Result<String, Error> {
    clip::line(Glyphs::for_text(text)?, 0, width)
}
