mod kill;

use std::ops::{Range, RangeInclusive};

use crate::buffer::{
    Buffer, Span, Stamp,
    edit::{Error, Point},
};
use crate::coordinates::{Coordinates, Glyph};

/// One owned block, separate from linear kill text. Width is the requested
/// display span; a touched wide cluster can make a row wider than that span.
pub struct Rectangle {
    rows: Vec<String>,
    width: usize,
}

impl Rectangle {
    pub fn rows(&self) -> &[String] {
        &self.rows
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub(crate) fn copy(buffer: &Buffer, selection: &Selection) -> Result<Self, Error> {
        let mut rows = Vec::new();
        for row in selection.rows.clone() {
            let mut text = String::new();
            for part in selection.parts(buffer, row)? {
                part?.append(buffer, &mut text)?;
            }
            rows.push(text);
        }
        Ok(Self {
            rows,
            width: selection.width(),
        })
    }
}

/// Geometry belongs to one source revision; rows include both endpoints and
/// columns are ordered independently of scalar order and viewport clipping.
pub(crate) struct Selection {
    stamp: Stamp,
    rows: RangeInclusive<usize>,
    columns: Range<usize>,
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
        let a = coordinates.to_cell(crate::coordinates::Scalar(region.range.start))?;
        let b = coordinates.to_cell(crate::coordinates::Scalar(region.range.end))?;
        Ok(Some(Self {
            stamp: region.stamp,
            rows: a.row..=b.row,
            columns: a.column.min(b.column)..a.column.max(b.column),
        }))
    }

    pub(crate) fn width(&self) -> usize {
        self.columns.end - self.columns.start
    }

    pub(crate) fn columns(
        &self,
        buffer: &Buffer,
        row: usize,
    ) -> Result<Option<Range<usize>>, Error> {
        if !self.rows.contains(&row) {
            return Ok(None);
        }
        let mut columns: Option<Range<usize>> = None;
        for part in self.parts(buffer, row)? {
            let part = part?;
            columns = Some(columns.map_or(part.columns.clone(), |old| old.start..part.columns.end));
        }
        Ok(columns)
    }

    fn parts<'a>(
        &'a self,
        buffer: &'a Buffer,
        row: usize,
    ) -> Result<impl Iterator<Item = Result<Part, Error>> + 'a, Error> {
        buffer.slice(&Span {
            stamp: self.stamp,
            range: 0..0,
        })?;
        let glyphs = Coordinates::new(buffer)?.glyphs(row)?;
        Ok(glyphs
            .take_while(|glyph| {
                glyph
                    .as_ref()
                    .map_or(true, |glyph| glyph.columns.start < self.columns.end)
            })
            .filter_map(|glyph| self.part(buffer, glyph).transpose()))
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
        let partial_tab = columns != glyph.columns && buffer.slice(&source)?.as_str() == Some("\t");
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
            spaces: partial_tab.then_some(columns.end - columns.start),
            columns: if partial_tab { columns } else { glyph.columns },
        }))
    }
}

struct Part {
    source: Span,
    columns: Range<usize>,
    spaces: Option<usize>,
    padding: usize,
    prefix: usize,
}

impl Part {
    fn append(&self, buffer: &Buffer, text: &mut String) -> Result<(), Error> {
        if let Some(cells) = self.spaces {
            text.extend(std::iter::repeat_n(' ', cells));
        } else {
            for chunk in buffer.chunks(&self.source)? {
                text.push_str(chunk);
            }
        }
        Ok(())
    }
}
