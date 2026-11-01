mod columns;
mod kill;
mod yank;
use columns::Columns;

use std::ops::{Range, RangeInclusive};

use crate::buffer::{
    Buffer, Span, Stamp,
    edit::{Error, Point},
};
use crate::coordinates::{Coordinates, Glyph};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Capture {
    Copy,
    Kill,
}

/// One native block with its source coordinate policy. Width is the requested
/// column span; display-cell edges can include a wider touched source cluster.
#[derive(Clone)]
pub struct Rectangle {
    rows: Vec<String>,
    width: usize,
    columns: Columns,
}

impl Rectangle {
    pub(crate) fn repeated(&self, count: usize) -> Result<Self, Error> {
        let bytes = self
            .rows
            .iter()
            .try_fold(0usize, |size, row| size.checked_add(row.len()))
            .and_then(|size| size.checked_mul(count))
            .ok_or(Error::Capacity)?;
        if count > 1 && bytes > crate::kill::EXPANSION_BYTES {
            return Err(Error::Capacity);
        }
        Ok(Self {
            rows: self.rows.iter().map(|row| row.repeat(count)).collect(),
            width: self.width.checked_mul(count).ok_or(Error::Capacity)?,
            columns: self.columns,
        })
    }

    pub fn rows(&self) -> &[String] {
        &self.rows
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub(crate) fn capture(
        buffer: &Buffer,
        selection: &Selection,
        capture: Capture,
    ) -> Result<Self, Error> {
        let mut rows = Vec::new();
        for row in selection.rows.clone() {
            let mut text = String::new();
            for part in selection.parts(buffer, row, None)? {
                part?.append(buffer, &mut text, capture)?;
            }
            rows.push(text);
        }
        Ok(Self {
            rows,
            width: selection.width(),
            columns: selection.policy,
        })
    }
}

/// Geometry belongs to one source revision; rows include both endpoints and
/// columns are ordered independently of scalar order and viewport clipping.
pub(crate) struct Selection {
    stamp: Stamp,
    rows: RangeInclusive<usize>,
    columns: Range<usize>,
    policy: Columns,
}

impl Selection {
    pub(crate) fn new(buffer: &Buffer, point: Point) -> Result<Option<Self>, Error> {
        let Some(region) = buffer.region(point)? else {
            return Ok(None);
        };
        if !buffer.rectangle_active() {
            return Ok(None);
        }
        let coordinates = Coordinates::new(buffer)?;
        let policy = if buffer.visual() == Some(crate::buffer::Visual::Block) {
            Columns::Grapheme
        } else {
            Columns::Display
        };
        let a = policy.cell(&coordinates, buffer.mark().ok_or(Error::Mark)?)?;
        let b = policy.cell(&coordinates, point.position)?;
        let end = a
            .column
            .max(b.column)
            .checked_add(usize::from(policy == Columns::Grapheme))
            .ok_or(Error::Capacity)?;
        Ok(Some(Self {
            stamp: region.stamp,
            rows: a.row.min(b.row)..=a.row.max(b.row),
            columns: a.column.min(b.column)..end,
            policy,
        }))
    }

    pub(crate) fn dimensions(&self) -> (usize, usize) {
        (self.rows.end() - self.rows.start() + 1, self.width())
    }

    pub(crate) fn at(
        buffer: &Buffer,
        point: Point,
        rows: usize,
        width: usize,
    ) -> Result<Self, Error> {
        buffer.slice(&Span {
            stamp: point.stamp,
            range: point.position.0..point.position.0,
        })?;
        let start = Coordinates::new(buffer)?.grapheme_cell(point.position)?;
        let last = start
            .row
            .saturating_add(rows.saturating_sub(1))
            .min(buffer.len_lines() - 1);
        Ok(Self {
            stamp: point.stamp,
            rows: start.row..=last,
            columns: start.column..start.column.checked_add(width).ok_or(Error::Capacity)?,
            policy: Columns::Grapheme,
        })
    }

    pub(crate) fn width(&self) -> usize {
        self.columns.end - self.columns.start
    }

    pub(crate) fn columns(
        &self,
        buffer: &Buffer,
        row: usize,
        visible: Range<usize>,
    ) -> Result<Option<Range<usize>>, Error> {
        if !self.rows.contains(&row) {
            return Ok(None);
        }
        let mut columns: Option<Range<usize>> = None;
        for part in self.parts(buffer, row, Some(visible))? {
            let part = part?;
            columns = Some(columns.map_or(part.columns.clone(), |old| old.start..part.columns.end));
        }
        Ok(columns)
    }

    fn parts<'a>(
        &'a self,
        buffer: &'a Buffer,
        row: usize,
        visible: Option<Range<usize>>,
    ) -> Result<impl Iterator<Item = Result<Part, Error>> + 'a, Error> {
        buffer.slice(&Span {
            stamp: self.stamp,
            range: 0..0,
        })?;
        let columns = self.visible_columns(visible);
        let glyphs = Coordinates::new(buffer)?.glyphs_from_column(crate::coordinates::Cell {
            row,
            column: if self.policy == Columns::Display {
                columns.start
            } else {
                0
            },
        })?;
        let end = columns.end;
        Ok(glyphs
            .enumerate()
            .take_while(move |(index, glyph)| {
                if self.policy == Columns::Grapheme {
                    return *index < end;
                }
                glyph
                    .as_ref()
                    .map_or(true, |glyph| glyph.columns.start < end)
            })
            .filter_map(|(index, glyph)| self.selected_part(buffer, index, glyph).transpose()))
    }

    fn visible_columns(&self, visible: Option<Range<usize>>) -> Range<usize> {
        if self.policy == Columns::Grapheme {
            return self.columns.clone();
        }
        visible.map_or(self.columns.clone(), |visible| {
            self.columns.start.max(visible.start)..self.columns.end.min(visible.end)
        })
    }

    fn selected_part(
        &self,
        buffer: &Buffer,
        index: usize,
        glyph: Result<Glyph<'_>, crate::coordinates::Error>,
    ) -> Result<Option<Part>, Error> {
        if self.policy == Columns::Display {
            return self.part(buffer, glyph);
        }
        let glyph = glyph?;
        if index < self.columns.start {
            return Ok(None);
        }
        Ok(Some(Part {
            source: buffer.span(glyph.source.start.0..glyph.source.end.0)?,
            columns: glyph.columns,
            tab: false,
            padding: 0,
            prefix: 0,
        }))
    }

    fn part(
        &self,
        buffer: &Buffer,
        glyph: Result<Glyph<'_>, crate::coordinates::Error>,
    ) -> Result<Option<Part>, Error> {
        let glyph = glyph?;
        let columns =
            glyph.columns.start.max(self.columns.start)..glyph.columns.end.min(self.columns.end);
        if columns.is_empty() {
            return Ok(None);
        }
        let source = buffer.span(glyph.source.start.0..glyph.source.end.0)?;
        let tab = buffer.slice(&source)?.as_str() == Some("\t");
        let partial_tab = tab && columns != glyph.columns;
        Ok(Some(Part {
            padding: if partial_tab {
                (glyph.columns.end - glyph.columns.start) - (columns.end - columns.start)
            } else {
                0
            },
            prefix: if partial_tab {
                columns.start - glyph.columns.start
            } else {
                0
            },
            source,
            tab,
            columns: if partial_tab { columns } else { glyph.columns },
        }))
    }
}

struct Part {
    source: Span,
    columns: Range<usize>,
    tab: bool,
    padding: usize,
    prefix: usize,
}

impl Part {
    fn append(&self, buffer: &Buffer, text: &mut String, capture: Capture) -> Result<(), Error> {
        // Partial tabs leave unselected padding; copy also expands complete
        // tabs. A kill retains complete source tabs for destination-side stops.
        if self.tab && (capture == Capture::Copy || self.padding > 0) {
            text.extend(std::iter::repeat_n(
                ' ',
                self.columns.end - self.columns.start,
            ));
        } else {
            for chunk in buffer.chunks(&self.source)? {
                text.push_str(chunk);
            }
        }
        Ok(())
    }
}
