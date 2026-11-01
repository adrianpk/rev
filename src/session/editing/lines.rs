use super::Error;
use crate::buffer::{Span, edit::Replacement};
use crate::command::editing::Action;
use crate::config::IndentStyle;
use crate::coordinates::{Cell, Coordinates, Scalar};
use crate::session::Session;

impl Session {
    pub(super) fn shift_lines(&mut self, source: &Span, action: Action) -> Result<bool, Error> {
        let mut edits = Vec::new();
        for row in self.source_line_rows(source)? {
            if let Some(edit) = self.shift_line(row, action)? {
                edits.push(edit);
            }
        }
        Ok(self.replace_many(&edits, None)?)
    }

    fn source_line_rows(&self, source: &Span) -> Result<std::ops::RangeInclusive<usize>, Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let first = coordinates.to_line(Scalar(source.range.start))?.row;
        let end = if source.range.is_empty() {
            Scalar(source.range.end)
        } else {
            coordinates.previous(Scalar(source.range.end))?
        };
        let last = coordinates.to_line(end)?.row;
        Ok(first..=last)
    }

    fn shift_line(&self, row: usize, action: Action) -> Result<Option<Replacement>, Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let start = coordinates.from_cell(Cell { row, column: 0 })?;
        if action == Action::Indent {
            let text = match buffer.policy().indent_style {
                IndentStyle::Tabs => "\t".into(),
                IndentStyle::Spaces => " ".repeat(buffer.policy().indent_width),
            };
            return Ok(Some(Replacement {
                source: buffer.span(start.0..start.0)?,
                text,
            }));
        }
        self.outdent_line(row, start)
    }

    fn outdent_line(&self, row: usize, start: Scalar) -> Result<Option<Replacement>, Error> {
        let buffer = self.buffer()?;
        let policy = buffer.policy();
        let unit = indentation_unit(policy);
        let mut end = start;
        let mut remainder = 0;
        for glyph in Coordinates::new(buffer)?.glyphs(row)? {
            let glyph = glyph?;
            if !glyph.text.chars().all(|c| c == ' ') {
                break;
            }
            end = glyph.source.end;
            if glyph.columns.end >= unit {
                remainder = glyph.columns.end - unit;
                break;
            }
        }
        if start == end {
            return Ok(None);
        }
        Ok(Some(Replacement {
            source: buffer.span(start.0..end.0)?,
            text: " ".repeat(remainder),
        }))
    }

    pub(super) fn join_lines(&mut self, count: usize) -> Result<bool, Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let row = coordinates.to_line(self.view().point.position)?.row;
        let last = row
            .saturating_add(count.saturating_sub(1))
            .min(buffer.len_lines() - 1);
        let edits = self.joined_edits(row..last)?;
        let cursor = edits.first().map(|e| Scalar(e.source.range.start));
        Ok(self.replace_many(&edits, cursor)?)
    }

    fn joined_edits(&self, rows: std::ops::Range<usize>) -> Result<Vec<Replacement>, Error> {
        let mut edits = Vec::new();
        let mut tail = self.line_tail(rows.start)?;
        for current in rows {
            let (range, empty) = self.join_range(current)?;
            let space = !empty && tail == Some(false);
            edits.push(Replacement {
                source: self.buffer()?.span(range)?,
                text: if space { " ".into() } else { String::new() },
            });
            if !empty {
                tail = self.line_tail(current + 1)?;
            }
        }
        Ok(edits)
    }

    fn join_range(&self, row: usize) -> Result<(std::ops::Range<usize>, bool), Error> {
        let coordinates = Coordinates::new(self.buffer()?)?;
        let start = coordinates.from_cell(Cell { row, column: 0 })?;
        let start = coordinates.line_end(start)?;
        let next = coordinates.from_cell(Cell {
            row: row + 1,
            column: 0,
        })?;
        let first =
            coordinates.navigation(next, crate::command::navigation::Target::FirstText, 1)?;
        Ok((start.0..first.0, first == coordinates.line_end(next)?))
    }

    fn line_tail(&self, row: usize) -> Result<Option<bool>, Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let start = coordinates.from_cell(Cell { row, column: 0 })?;
        let end = coordinates.line_end(start)?;
        if start == end {
            return Ok(None);
        }
        let previous = coordinates.previous(end)?;
        Ok(Some(
            buffer
                .slice(&buffer.span(previous.0..end.0)?)?
                .chars()
                .all(char::is_whitespace),
        ))
    }
}

fn indentation_unit(policy: &crate::config::Editor) -> usize {
    if policy.indent_style == IndentStyle::Tabs {
        policy.tab_width
    } else {
        policy.indent_width
    }
}
