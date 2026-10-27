use std::borrow::Cow;
use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::{
    Buffer, Span,
    edit::{Error, Replacement},
};
use crate::coordinates::{Coordinates, Glyph, Scalar};

/// Explicit conversion of existing selected whitespace under the buffer policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Whitespace {
    Tabify,
    Untabify,
}

impl Whitespace {
    pub fn name(self) -> &'static str {
        match self {
            Self::Tabify => "tabify",
            Self::Untabify => "untabify",
        }
    }
}

impl Buffer {
    /// Derive disjoint whitespace replacements from one checked source revision.
    /// Glyph projection supplies true columns, including the unselected prefix.
    pub(crate) fn whitespace(
        &self,
        source: &Span,
        operation: Whitespace,
    ) -> Result<Vec<Replacement>, Error> {
        self.slice(source)?;
        let coordinates = Coordinates::new(self)?;
        let first = coordinates.to_line(Scalar(source.range.start))?.row;
        let last = coordinates.to_line(Scalar(source.range.end))?.row;
        let mut conversion = Conversion {
            buffer: self,
            source,
            operation,
            edits: Vec::new(),
        };
        for row in first..=last {
            conversion.line(&coordinates, row)?;
        }
        Ok(conversion.edits)
    }
}

struct Conversion<'a> {
    buffer: &'a Buffer,
    source: &'a Span,
    operation: Whitespace,
    edits: Vec<Replacement>,
}

struct Run {
    source: Range<usize>,
    columns: Range<usize>,
    ends_tab: bool,
}

impl Conversion<'_> {
    fn line(&mut self, coordinates: &Coordinates<'_>, row: usize) -> Result<(), Error> {
        let mut run = None;
        for glyph in coordinates.glyphs(row)? {
            let glyph = glyph?;
            if glyph.source.start.0 >= self.source.range.end {
                return self.flush(&mut run, Some(&glyph));
            }
            if self.selected(&glyph) {
                self.glyph(&mut run, glyph)?;
            }
        }
        self.flush(&mut run, None)
    }

    fn selected(&self, glyph: &Glyph<'_>) -> bool {
        glyph.source.start.0 >= self.source.range.start
            && glyph.source.end.0 <= self.source.range.end
    }

    fn glyph(&mut self, run: &mut Option<Run>, glyph: Glyph<'_>) -> Result<(), Error> {
        let source = self.buffer.span(glyph.source.start.0..glyph.source.end.0)?;
        let original = self.buffer.slice(&source)?;
        if matches!(original.as_str(), Some(" " | "\t")) {
            let run = run.get_or_insert_with(|| Run {
                source: source.range.clone(),
                columns: glyph.columns.clone(),
                ends_tab: false,
            });
            run.source.end = source.range.end;
            run.columns.end = glyph.columns.end;
            run.ends_tab = original.as_str() == Some("\t");
        } else {
            self.flush(run, Some(&glyph))?;
        }
        Ok(())
    }

    fn flush(&mut self, run: &mut Option<Run>, next: Option<&Glyph<'_>>) -> Result<(), Error> {
        let Some(run) = run.take() else {
            return Ok(());
        };
        let padding = if self.operation == Whitespace::Untabify && run.ends_tab {
            next.map(|glyph| joining_padding(self.buffer, glyph))
                .transpose()?
                .unwrap_or(0)
        } else {
            0
        };
        if let Some(edit) = run.replacement(self.buffer, self.operation, padding)? {
            self.edits.push(edit);
        }
        Ok(())
    }
}

impl Run {
    fn replacement(
        self,
        buffer: &Buffer,
        operation: Whitespace,
        padding: usize,
    ) -> Result<Option<Replacement>, Error> {
        let source = buffer.span(self.source.clone())?;
        let original = buffer.slice(&source)?;
        let text = match operation {
            Whitespace::Tabify => self.tabs(buffer.policy().tab_width)?,
            Whitespace::Untabify => self.spaces(padding)?,
        };
        if original.chars().eq(text.chars()) {
            return Ok(None);
        }
        if operation == Whitespace::Tabify && text.len() >= self.source.len() {
            return Ok(None);
        }
        Ok(Some(Replacement { source, text }))
    }

    fn spaces(&self, padding: usize) -> Result<String, Error> {
        let count = self
            .columns
            .len()
            .checked_add(padding)
            .ok_or(Error::Capacity)?;
        let mut text = String::new();
        text.try_reserve(count).map_err(|_| Error::Capacity)?;
        text.extend(std::iter::repeat_n(' ', count));
        Ok(text)
    }

    fn tabs(&self, width: usize) -> Result<String, Error> {
        let mut text = String::new();
        // The minimal representation cannot exceed the existing ASCII run.
        text.try_reserve(self.source.len())
            .map_err(|_| Error::Capacity)?;
        let mut column = self.columns.start;
        while column < self.columns.end {
            let step = width - column % width;
            if step > 1 && step <= self.columns.end - column {
                text.push('\t');
                column += step;
            } else {
                text.push(' ');
                column += 1;
            }
        }
        Ok(text)
    }
}

// A tab separates a leading zero-width cluster that rendering gives one visible
// cell. After expansion it can join the final space; retain that presentation
// cell as a space anchor without changing the following Unicode source bytes.
fn joining_padding(buffer: &Buffer, glyph: &Glyph<'_>) -> Result<usize, Error> {
    let source = buffer.span(glyph.source.start.0..glyph.source.end.0)?;
    let original = buffer.slice(&source)?;
    let text = original
        .as_str()
        .map(Cow::Borrowed)
        .unwrap_or_else(|| Cow::Owned(original.to_string()));
    if text.width() != 0 {
        return Ok(0);
    }
    let first = text
        .chars()
        .next()
        .ok_or(crate::coordinates::Error::Segmentation)?;
    let probe = format!(" {first}");
    Ok(usize::from(probe.graphemes(true).count() == 1))
}
