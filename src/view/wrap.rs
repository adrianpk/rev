mod cache;
pub(super) use cache::Cache;
use std::collections::VecDeque;
use std::ops::Range;

use crate::buffer::{Buffer, Stamp};
use crate::coordinates::{Cell, Coordinates, Glyph, GlyphCheckpoint, Glyphs, Scalar};

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
    // Includes break lookahead and the complete observed line terminator.
    observed_end: Scalar,
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
    glyphs: Glyphs<'a>,
    lookahead: Option<Glyph<'a>>,
    exhausted: bool,
    pending: VecDeque<Glyph<'a>>,
    line: usize,
    width: usize,
    end: Scalar,
    after_line: Scalar,
    column: usize,
    done: bool,
}

pub(super) struct Continuation {
    stamp: Stamp,
    glyphs: GlyphCheckpoint,
    lookahead: Option<Glyph<'static>>,
    exhausted: bool,
    pending: VecDeque<Glyph<'static>>,
    pub(super) line: usize,
    width: usize,
    pub(super) end: Scalar,
    after_line: Scalar,
    column: usize,
    done: bool,
}

pub(crate) struct Budget {
    remaining: usize,
}

impl Budget {
    pub(crate) const GLYPHS: usize = 4096;

    pub(crate) fn new() -> Self {
        Self {
            remaining: Self::GLYPHS,
        }
    }

    fn charge(&mut self) -> bool {
        if self.remaining == 0 {
            return false;
        }
        self.remaining -= 1;
        true
    }
}

pub(super) enum Step {
    Fragment(Fragment),
    Pending,
    Finished,
}

impl<'a> Line<'a> {
    pub(super) fn new(buffer: &'a Buffer, line: usize, width: u16) -> Result<Self, Error> {
        let coordinates = Coordinates::new(buffer)?;
        Ok(Self {
            stamp: buffer.stamp(),
            glyphs: coordinates.glyphs(line)?,
            lookahead: None,
            exhausted: false,
            pending: VecDeque::new(),
            line,
            width: usize::from(width),
            end: coordinates.from_cell(Cell {
                row: line,
                column: 0,
            })?,
            after_line: Self::after_line(buffer, &coordinates, line)?,
            column: 0,
            done: width == 0,
        })
    }

    pub(super) fn resume(
        buffer: &'a Buffer,
        line: usize,
        width: u16,
        start: Scalar,
        column: usize,
    ) -> Result<Self, Error> {
        let coordinates = Coordinates::new(buffer)?;
        Ok(Self {
            stamp: buffer.stamp(),
            glyphs: coordinates.glyphs_at(line, start, column)?,
            lookahead: None,
            exhausted: false,
            pending: VecDeque::new(),
            line,
            width: usize::from(width),
            end: start,
            after_line: Self::after_line(buffer, &coordinates, line)?,
            column,
            done: width == 0,
        })
    }

    fn continue_from(buffer: &'a Buffer, at: Continuation) -> Result<Self, Error> {
        Ok(Self {
            stamp: at.stamp,
            glyphs: Coordinates::new(buffer)?.resume_glyphs(at.line, at.glyphs)?,
            lookahead: at.lookahead,
            exhausted: at.exhausted,
            pending: at.pending,
            line: at.line,
            width: at.width,
            end: at.end,
            after_line: at.after_line,
            column: at.column,
            done: at.done,
        })
    }

    fn suspend(self) -> Continuation {
        Continuation {
            stamp: self.stamp,
            glyphs: self.glyphs.checkpoint(),
            lookahead: self.lookahead.map(Glyph::into_owned),
            exhausted: self.exhausted,
            pending: self.pending.into_iter().map(Glyph::into_owned).collect(),
            line: self.line,
            width: self.width,
            end: self.end,
            after_line: self.after_line,
            column: self.column,
            done: self.done,
        }
    }

    fn after_line(
        buffer: &Buffer,
        coordinates: &Coordinates<'_>,
        line: usize,
    ) -> Result<Scalar, Error> {
        if line + 1 < buffer.len_lines() {
            Ok(coordinates.from_cell(Cell {
                row: line + 1,
                column: 0,
            })?)
        } else {
            Ok(Scalar(buffer.len_chars()))
        }
    }

    pub(super) fn next(&mut self) -> Result<Option<Fragment>, Error> {
        let mut budget = Budget {
            remaining: usize::MAX,
        };
        match self.advance(&mut budget)? {
            Step::Fragment(fragment) => Ok(Some(fragment)),
            Step::Finished => Ok(None),
            Step::Pending => unreachable!("unbounded reference iteration exhausted usize"),
        }
    }

    fn advance(&mut self, budget: &mut Budget) -> Result<Step, Error> {
        if self.done {
            return Ok(Step::Finished);
        }
        let start = self
            .pending
            .front()
            .map_or(self.column, |g| g.columns.start);
        loop {
            if self.lookahead.is_none() && !self.exhausted {
                if !budget.charge() {
                    return Ok(Step::Pending);
                }
                self.lookahead = self.glyphs.next().transpose()?;
                self.exhausted = self.lookahead.is_none();
            }
            let Some(glyph) = &self.lookahead else { break };
            if glyph.columns.end - start > self.width && !self.pending.is_empty() {
                break;
            }
            self.pending
                .push_back(self.lookahead.take().expect("observed lookahead"));
        }
        let more = self.lookahead.is_some();
        let count = if more {
            self.pending
                .iter()
                .rposition(|glyph| glyph.text.chars().all(|c| c == ' '))
                .map_or(self.pending.len(), |index| index + 1)
        } else {
            self.pending.len()
        };
        let observed_end = self.lookahead.as_ref().map_or_else(
            || self.pending.back().map_or(self.end, |g| g.source.end),
            |g| g.source.end,
        );
        // EOL also observes the complete terminator. Removing only LF from
        // CRLF can change the body's final glyph and join the following row.
        let observed_end = if more { observed_end } else { self.after_line };
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
        Ok(Step::Fragment(Fragment {
            stamp: self.stamp,
            line: self.line,
            source: source..self.end,
            columns: start..self.column,
            text,
            end_of_line: !more && self.pending.is_empty(),
            observed_end,
        }))
    }
}

impl View {
    pub(crate) fn cancel_layout(&self) {
        self.wrap_cache.cancel_work();
    }

    /// Prepare the source anchors needed by a frame without monopolizing its
    /// host. The same builder/segmentation state resumes on the next deadline.
    pub(crate) fn prepare_layout(
        &self,
        buffer: &Buffer,
        size: super::Size,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        if !buffer.soft_wrap() || !size.usable() {
            return Ok(true);
        }
        let width = self.text_width(buffer, size)?;
        if width == 0 {
            return Ok(true);
        }
        buffer.slice(&crate::buffer::Span {
            stamp: self.point.stamp,
            range: self.point.position.0..self.point.position.0,
        })?;
        let coordinates = Coordinates::new(buffer)?;
        let row = coordinates.to_line(self.point.position)?.row;
        if !self.prepare_fragment(buffer, width, row, self.point.position, budget)? {
            return Ok(false);
        }
        let Some(row) = self.source_at(buffer, self.projected_top(buffer)) else {
            return Ok(true);
        };
        let start = coordinates.from_cell(Cell { row, column: 0 })?;
        let end = coordinates.line_end(start)?;
        let target = if row == self.top {
            self.wrap_start.max(start).min(end)
        } else {
            start
        };
        self.prepare_fragment(buffer, width, row, target, budget)
    }

    fn prepare_fragment(
        &self,
        buffer: &Buffer,
        width: u16,
        row: usize,
        point: Scalar,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let checkpoint = self.wrap_cache.before(buffer, width, row, point);
        if checkpoint
            .as_ref()
            .is_some_and(|f| f.contains(point, width))
        {
            return Ok(true);
        }
        let mut line = if let Some(at) = self.wrap_cache.take_work(buffer, width, row, point) {
            Line::continue_from(buffer, at)?
        } else if let Some(fragment) = checkpoint {
            Line::resume(
                buffer,
                row,
                width,
                fragment.source.start,
                fragment.columns.start,
            )?
        } else {
            Line::new(buffer, row, width)?
        };
        let coordinates = Coordinates::new(buffer)?;
        loop {
            match line.advance(budget)? {
                Step::Fragment(fragment) => {
                    coordinates.remember_cell(
                        fragment.source.start,
                        Cell {
                            row,
                            column: fragment.columns.start,
                        },
                    );
                    coordinates.remember_cell(
                        fragment.source.end,
                        Cell {
                            row,
                            column: fragment.columns.end,
                        },
                    );
                    let found = fragment.contains(point, width);
                    self.wrap_cache.remember(buffer, width, fragment);
                    if found {
                        return Ok(true);
                    }
                }
                Step::Pending => {
                    self.wrap_cache.suspend(buffer, width, line.suspend());
                    return Ok(false);
                }
                Step::Finished => return Err(crate::coordinates::Error::Scalar(point).into()),
            }
        }
    }

    /// Derive visible rows from checked fragment checkpoints. Cache disposal
    /// changes recomputation cost only, never source visibility or identity.
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
        let Some(source) = self.source_at(buffer, projected) else {
            return Ok(result);
        };
        let coordinates = Coordinates::new(buffer)?;
        let start = coordinates.from_cell(Cell {
            row: source,
            column: 0,
        })?;
        let end = coordinates.line_end(start)?;
        let target = if source == self.top {
            self.wrap_start.max(start).min(end)
        } else {
            start
        };
        let mut current = self.line_fragment(buffer, width, source, target)?;
        loop {
            let finished = current.end_of_line
                && (current.source.is_empty() || current.painted_width(width) < width);
            result.push(current.clone());
            if result.len() == usize::from(rows) {
                break;
            }
            if finished {
                projected = projected
                    .checked_add(1)
                    .ok_or(crate::coordinates::Error::Overflow)?;
                let Some(source) = self.source_at(buffer, projected) else {
                    break;
                };
                let Some(next) = Line::new(buffer, source, width)?.next()? else {
                    break;
                };
                self.wrap_cache.remember(buffer, width, next.clone());
                current = next;
            } else {
                current = self.next_fragment(buffer, width, &current)?;
            }
        }
        Ok(result)
    }

    pub(super) fn line_fragment(
        &self,
        buffer: &Buffer,
        width: u16,
        row: usize,
        point: Scalar,
    ) -> Result<Fragment, Error> {
        let checkpoint = self.wrap_cache.before(buffer, width, row, point);
        if let Some(fragment) = &checkpoint
            && fragment.contains(point, width)
        {
            return Ok(fragment.clone());
        }
        let mut line = if let Some(fragment) = checkpoint {
            Line::resume(
                buffer,
                row,
                width,
                fragment.source.start,
                fragment.columns.start,
            )?
        } else {
            Line::new(buffer, row, width)?
        };
        while let Some(fragment) = line.next()? {
            let found = fragment.contains(point, width);
            self.wrap_cache.remember(buffer, width, fragment.clone());
            if found {
                return Ok(fragment);
            }
        }
        Err(crate::coordinates::Error::Scalar(point).into())
    }

    pub(super) fn next_fragment(
        &self,
        buffer: &Buffer,
        width: u16,
        current: &Fragment,
    ) -> Result<Fragment, Error> {
        if let Some(fragment) =
            self.wrap_cache
                .exact(buffer, width, current.line, current.source.end)
        {
            return Ok(fragment);
        }
        let mut line = Line::resume(
            buffer,
            current.line,
            width,
            current.source.end,
            current.columns.end,
        )?;
        let next = line.next()?.ok_or(crate::coordinates::Error::Overflow)?;
        self.wrap_cache.remember(buffer, width, next.clone());
        Ok(next)
    }

    pub(super) fn reveal_wrapped(
        &mut self,
        buffer: &Buffer,
        width: u16,
        rows: u16,
    ) -> Result<(), Error> {
        if self
            .wrapped_rows(buffer, width, rows)?
            .iter()
            .any(|f| f.contains(self.point.position, width))
        {
            return Ok(());
        }
        let mut top = self.visual_fragment(buffer, width)?;
        for _ in 1..rows {
            let Some(previous) = self.adjacent_fragment(buffer, width, &top, true)? else {
                break;
            };
            top = previous;
        }
        self.top = top.line;
        self.wrap_start = top.source.start;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{Buffers, edit::Replacement};

    fn reference(buffer: &Buffer, width: u16) -> Vec<Fragment> {
        let mut result = Vec::new();
        for row in 0..buffer.len_lines() {
            let mut line = Line::new(buffer, row, width).unwrap();
            while let Some(fragment) = line.next().unwrap() {
                result.push(fragment);
            }
        }
        result
    }

    // A restarted row must be identical to continuous layout, including the
    // lookahead that decides word breaks and Unicode context at chunk seams.
    #[test]
    fn fragment_restarts() {
        let mut buffers = Buffers::new().unwrap();
        for source in [
            "one two three longtoken".repeat(100),
            "a\t界e\u{301}👩‍💻 🇦🇧🇨 ".repeat(100),
            "🇦🇧".repeat(600),
            "a".repeat(100),
        ] {
            let id = buffers.create(&source).unwrap();
            let b = buffers.get(id).unwrap();
            for width in [1, 3, 8, 13, 80] {
                for expected in reference(b, width) {
                    let actual =
                        Line::resume(b, 0, width, expected.source.start, expected.columns.start)
                            .unwrap()
                            .next()
                            .unwrap()
                            .unwrap();
                    assert_eq!(actual, expected, "{width}/{}", expected.source.start.0);
                }
            }
        }
    }

    // The break decision reads beyond its returned source span. Shortening a
    // following token must invalidate that earlier fragment, including on undo.
    #[test]
    fn break_lookahead() {
        let mut buffers = Buffers::new().unwrap();
        let source = format!("{}one longtoken", "alpha beta gamma ".repeat(20));
        let id = buffers.create(&source).unwrap();
        let b = buffers.get_mut(id).unwrap();
        let mut view = View::new(b);
        assert_eq!(view.wrapped_rows(b, 8, 1000).unwrap(), reference(b, 8));
        let edit = Replacement {
            source: b.span(b.len_chars() - 9..b.len_chars()).unwrap(),
            text: "x".into(),
        };
        let mut points = [view.point];
        let prepared = b.prepare(&[edit], &points, None).unwrap();
        b.publish(prepared, &mut points).unwrap();
        view.point = points[0];
        assert_eq!(view.wrapped_rows(b, 8, 1000).unwrap(), reference(b, 8));
        b.navigate(crate::buffer::edit::History::Undo, &mut points, None)
            .unwrap();
        view.point = points[0];
        assert_eq!(view.wrapped_rows(b, 8, 1000).unwrap(), reference(b, 8));
    }

    // An EOL observation includes both CR and LF, even though neither is
    // painted. Deleting LF alone must discard the prior final fragment.
    #[test]
    fn terminator_invalidation() {
        let mut buffers = Buffers::new().unwrap();
        let source = format!("{}tail\r\nfollowing", "abc ".repeat(20));
        let lf = source.find('\n').unwrap();
        let id = buffers.create(&source).unwrap();
        let b = buffers.get_mut(id).unwrap();
        let mut view = View::new(b);
        assert_eq!(view.wrapped_rows(b, 8, 1000).unwrap(), reference(b, 8));
        let edit = Replacement {
            source: b.span(lf..lf + 1).unwrap(),
            text: String::new(),
        };
        let mut points = [view.point];
        let prepared = b.prepare(&[edit], &points, None).unwrap();
        b.publish(prepared, &mut points).unwrap();
        view.point = points[0];
        assert_eq!(view.wrapped_rows(b, 8, 1000).unwrap(), reference(b, 8));
    }

    // Suspending inside lookahead or a rope chunk must preserve the continuous
    // word/grapheme partition. Each batch charges at most its glyph allowance.
    #[test]
    fn cooperative_batches() {
        let mut buffers = Buffers::new().unwrap();
        for source in [
            "one two longtoken a\t界e\u{301}👩‍💻 ".repeat(150),
            "🇦🇧".repeat(600),
        ] {
            let id = buffers.create(&source).unwrap();
            let b = buffers.get(id).unwrap();
            for width in [1, 13, 4097] {
                let expected = reference(b, width);
                for allowance in [1, 7, 256] {
                    let mut actual = Vec::new();
                    let mut line = Line::new(b, 0, width).unwrap();
                    let mut batches = 0;
                    loop {
                        let mut budget = Budget {
                            remaining: allowance,
                        };
                        loop {
                            match line.advance(&mut budget).unwrap() {
                                Step::Fragment(f) => actual.push(f),
                                Step::Pending => break,
                                Step::Finished => break,
                            }
                        }
                        batches += 1;
                        assert!(batches <= b.len_chars() + 2);
                        if line.done {
                            break;
                        }
                        assert_eq!(budget.remaining, 0);
                        line = Line::continue_from(b, line.suspend()).unwrap();
                    }
                    assert_eq!(actual, expected, "{width}/{allowance}");
                }
            }
        }
    }
}
