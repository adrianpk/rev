use std::ops::Range;

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
}

impl Conversion<'_> {
    fn line(&mut self, coordinates: &Coordinates<'_>, row: usize) -> Result<(), Error> {
        let mut run = None;
        for glyph in coordinates.glyphs(row)? {
            let glyph = glyph?;
            if glyph.source.start.0 >= self.source.range.end {
                break;
            }
            if self.selected(&glyph) {
                self.glyph(&mut run, glyph)?;
            }
        }
        self.flush(&mut run)
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
            });
            run.source.end = source.range.end;
            run.columns.end = glyph.columns.end;
        } else {
            self.flush(run)?;
        }
        Ok(())
    }

    fn flush(&mut self, run: &mut Option<Run>) -> Result<(), Error> {
        if let Some(run) = run.take()
            && let Some(edit) = run.replacement(self.buffer, self.operation)?
        {
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
    ) -> Result<Option<Replacement>, Error> {
        let source = buffer.span(self.source.clone())?;
        let original = buffer.slice(&source)?;
        let text = match operation {
            Whitespace::Tabify => self.tabs(buffer.policy().tab_width)?,
            Whitespace::Untabify => self.spaces()?,
        };
        if original.chars().eq(text.chars()) {
            return Ok(None);
        }
        if operation == Whitespace::Tabify && text.len() >= self.source.len() {
            return Ok(None);
        }
        Ok(Some(Replacement { source, text }))
    }

    fn spaces(&self) -> Result<String, Error> {
        let mut text = String::new();
        text.try_reserve(self.columns.len())
            .map_err(|_| Error::Capacity)?;
        text.extend(std::iter::repeat_n(' ', self.columns.len()));
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
