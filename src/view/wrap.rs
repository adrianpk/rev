use std::collections::VecDeque;
use std::ops::Range;

use crate::buffer::{Buffer, Stamp};
use crate::coordinates::{Cell, Coordinates, Glyph, Glyphs, Scalar};

use super::{Error, View};

/// A checked source observation at one pane width. Text contains presentation
/// cells only; the scalar span always addresses the unchanged canonical buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fragment {
    pub stamp: Stamp,
    pub line: usize,
    pub source: Range<Scalar>,
    pub columns: Range<usize>,
    pub text: String,
    pub end_of_line: bool,
}

impl Fragment {
    pub(crate) fn painted_width(&self, width: u16) -> u16 {
        (self.columns.end - self.columns.start).min(usize::from(width)) as u16
    }

    // Only a real source newline can select the remaining blank cells.
    pub(crate) fn highlight_width(&self, width: u16) -> u16 {
        if self.end_of_line {
            width
        } else {
            self.painted_width(width)
        }
    }

    pub(crate) fn contains(&self, point: Scalar, width: u16) -> bool {
        self.source.contains(&point)
            || self.end_of_line
                && point == self.source.end
                && self.columns.end - self.columns.start < usize::from(width)
    }
}

/// Streaming one-line layout retains at most one pane-width of lookahead.
/// Word fallback never consumes part of a grapheme, even in a one-cell pane.
pub(super) struct Line<'a> {
    stamp: Stamp,
    glyphs: std::iter::Peekable<Glyphs<'a>>,
    pending: VecDeque<Glyph<'a>>,
    line: usize,
    width: usize,
    end: Scalar,
    column: usize,
    done: bool,
}

impl<'a> Line<'a> {
    pub(super) fn new(buffer: &'a Buffer, line: usize, width: u16) -> Result<Self, Error> {
        let coordinates = Coordinates::new(buffer)?;
        Ok(Self {
            stamp: buffer.stamp(),
            glyphs: coordinates.glyphs(line)?.peekable(),
            pending: VecDeque::new(),
            line,
            width: usize::from(width),
            end: coordinates.from_cell(Cell {
                row: line,
                column: 0,
            })?,
            column: 0,
            done: width == 0,
        })
    }

    pub(super) fn next(&mut self) -> Result<Option<Fragment>, Error> {
        if self.done {
            return Ok(None);
        }
        let start = self
            .pending
            .front()
            .map_or(self.column, |g| g.columns.start);
        while let Some(next) = self.glyphs.peek() {
            match next {
                Ok(glyph) if glyph.columns.end - start <= self.width => {}
                Ok(_) if self.pending.is_empty() => {}
                Ok(_) => break,
                Err(error) => return Err((*error).into()),
            }
            if let Some(glyph) = self.glyphs.next() {
                self.pending.push_back(glyph?);
            }
        }
        let more = self.glyphs.peek().is_some();
        let count = if more {
            self.pending
                .iter()
                .rposition(|glyph| glyph.text.chars().all(|c| c == ' '))
                .map_or(self.pending.len(), |index| index + 1)
        } else {
            self.pending.len()
        };
        let source = self.pending.front().map_or(self.end, |g| g.source.start);
        let mut text = String::new();
        for glyph in self.pending.drain(..count) {
            self.end = glyph.source.end;
            self.column = glyph.columns.end;
            if glyph.columns.end - start <= self.width {
                text.push_str(&glyph.text);
            } else {
                text.extend(std::iter::repeat_n(' ', self.width));
            }
        }
        let cells = (self.column - start).min(self.width);
        text.extend(std::iter::repeat_n(' ', self.width - cells));
        // An exact-width EOL needs a final insertion cell on the next row.
        self.done = !more && self.pending.is_empty() && (count == 0 || cells < self.width);
        Ok(Some(Fragment {
            stamp: self.stamp,
            line: self.line,
            source: source..self.end,
            columns: start..self.column,
            text,
            end_of_line: !more && self.pending.is_empty(),
        }))
    }
}

impl View {
    /// Derive only the visible rows, beginning at a source-qualified anchor.
    /// No layout cache or document-wide row index survives this observation.
    pub fn wrapped_rows(
        &self,
        buffer: &Buffer,
        width: u16,
        rows: u16,
    ) -> Result<Vec<Fragment>, Error> {
        self.cell(buffer)?;
        let mut result = Vec::new();
        if width == 0 || rows == 0 {
            return Ok(result);
        }
        let mut projected = self.projected_top(buffer);
        while let Some(source) = self.source_at(buffer, projected) {
            let mut line = Line::new(buffer, source, width)?;
            while let Some(fragment) = line.next()? {
                if source == self.top
                    && fragment.source.end <= self.wrap_start
                    && !fragment.source.is_empty()
                    && !fragment.contains(self.wrap_start, width)
                {
                    continue;
                }
                result.push(fragment);
                if result.len() == usize::from(rows) {
                    return Ok(result);
                }
            }
            projected = projected
                .checked_add(1)
                .ok_or(crate::coordinates::Error::Overflow)?;
        }
        Ok(result)
    }

    pub(super) fn reveal_wrapped(
        &mut self,
        buffer: &Buffer,
        width: u16,
        rows: u16,
    ) -> Result<(), Error> {
        let point = self.cell(buffer)?;
        if self
            .wrapped_rows(buffer, width, rows)?
            .iter()
            .any(|f| f.line == point.row && f.contains(self.point.position, width))
        {
            return Ok(());
        }
        let target = self
            .project_row(buffer, point.row)
            .ok_or(crate::coordinates::Error::Overflow)?;
        let mut visible = VecDeque::new();
        for projected in target.saturating_sub(usize::from(rows) - 1)..=target {
            let source = self
                .source_at(buffer, projected)
                .ok_or(crate::coordinates::Error::Overflow)?;
            let mut line = Line::new(buffer, source, width)?;
            while let Some(fragment) = line.next()? {
                let found = source == point.row && fragment.contains(self.point.position, width);
                visible.push_back(fragment);
                if visible.len() > usize::from(rows) {
                    visible.pop_front();
                }
                if found {
                    if let Some(first) = visible.front() {
                        self.top = first.line;
                        self.wrap_start = first.source.start;
                    }
                    return Ok(());
                }
            }
        }
        Err(crate::coordinates::Error::Scalar(self.point.position).into())
    }
}
