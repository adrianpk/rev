use std::io::{self, Write};
use std::ops::Range;

use crossterm::{
    QueueableCommand,
    style::{Attribute, SetAttribute},
};

use crate::buffer::{Buffer, Span};
use crate::coordinates::{Cell, Coordinates, Glyphs, Scalar};
use crate::view::View;

use super::{Error, Row};

pub(super) struct Highlight {
    start: Cell,
    end: Cell,
}

impl Highlight {
    pub(super) fn new(buffer: &Buffer, view: &View, selected: bool) -> Result<Option<Self>, Error> {
        if !selected {
            return Ok(None);
        }
        let Some(region) = buffer.region(view.point)? else {
            return Ok(None);
        };
        Ok(Some(Self::from_span(buffer, &region)?))
    }

    pub(super) fn from_span(buffer: &Buffer, span: &Span) -> Result<Self, Error> {
        buffer.slice(span)?;
        let coordinates = Coordinates::new(buffer)?;
        let start = boundary(&coordinates, Scalar(span.range.start), false)?;
        let end = boundary(&coordinates, Scalar(span.range.end), true)?;
        Ok(Self {
            start: coordinates.to_cell(start)?,
            end: coordinates.to_cell(end)?,
        })
    }

    pub(super) fn row(&self, row: usize, left: usize, width: u16) -> Option<Range<u16>> {
        if row < self.start.row || row > self.end.row {
            return None;
        }
        let start = if row == self.start.row {
            self.start.column
        } else {
            0
        };
        // A selected line break highlights the remainder of its visible row.
        let end = if row == self.end.row {
            self.end.column
        } else {
            usize::MAX
        };
        let start = start.saturating_sub(left).min(usize::from(width));
        let end = end.saturating_sub(left).min(usize::from(width));
        // Both offsets are bounded by the admitted native width.
        (start < end).then_some(start as u16..end as u16)
    }
}

fn boundary(coordinates: &Coordinates<'_>, point: Scalar, forward: bool) -> Result<Scalar, Error> {
    if coordinates.is_boundary(point)? {
        return Ok(point);
    }
    Ok(if forward {
        coordinates.next(point)?
    } else {
        coordinates.previous(point)?
    })
}

impl Row {
    pub(super) fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        if self.selection.is_none() && self.search.is_none() {
            return writer.write_all(self.text.as_bytes());
        }
        let mut reversed = false;
        let mut underlined = false;
        for glyph in Glyphs::for_text(&self.text).map_err(io::Error::other)? {
            let glyph = glyph.map_err(io::Error::other)?;
            let selected = covers(&self.selection, glyph.columns.start);
            reverse(writer, selected, &mut reversed)?;
            underline(
                writer,
                covers(&self.search, glyph.columns.start),
                &mut underlined,
            )?;
            writer.write_all(glyph.text.as_bytes())?;
        }
        reverse(writer, false, &mut reversed)?;
        underline(writer, false, &mut underlined)
    }
}

fn covers(range: &Option<Range<u16>>, column: usize) -> bool {
    range
        .as_ref()
        .is_some_and(|range| usize::from(range.start) <= column && column < usize::from(range.end))
}

fn underline(writer: &mut impl Write, marked: bool, underlined: &mut bool) -> io::Result<()> {
    if marked != *underlined {
        let attribute = if marked {
            Attribute::Underlined
        } else {
            Attribute::NoUnderline
        };
        writer.queue(SetAttribute(attribute))?;
        *underlined = marked;
    }
    Ok(())
}

fn reverse(writer: &mut impl Write, selected: bool, reversed: &mut bool) -> io::Result<()> {
    if selected != *reversed {
        let attribute = if selected {
            Attribute::Reverse
        } else {
            Attribute::NoReverse
        };
        writer.queue(SetAttribute(attribute))?;
        *reversed = selected;
    }
    Ok(())
}
