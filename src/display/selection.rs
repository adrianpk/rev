use std::io::{self, Write};
use std::ops::Range;

use crossterm::{
    QueueableCommand,
    style::{Attribute, SetAttribute},
};

use crate::buffer::{Buffer, Span};
use crate::coordinates::{Cell, Coordinates, Glyphs, Scalar};
use crate::rectangle::Selection;
use crate::view::View;

use super::{Error, Row};

pub(super) struct Highlight {
    start: Cell,
    end: Cell,
    rectangle: Option<Selection>,
}

impl Highlight {
    pub(super) fn new(buffer: &Buffer, view: &View, selected: bool) -> Result<Option<Self>, Error> {
        if !selected {
            return Ok(None);
        }
        let Some(region) = buffer.region(view.point)? else {
            return Ok(None);
        };
        let mut highlight = Self::from_span(buffer, &region)?;
        highlight.rectangle = Selection::new(buffer, view.point)?;
        Ok(Some(highlight))
    }

    pub(super) fn from_span(buffer: &Buffer, span: &Span) -> Result<Self, Error> {
        buffer.slice(span)?;
        let coordinates = Coordinates::new(buffer)?;
        let start = boundary(&coordinates, Scalar(span.range.start), false)?;
        let end = boundary(&coordinates, Scalar(span.range.end), true)?;
        Ok(Self {
            rectangle: None,
            start: coordinates.to_cell(start)?,
            end: coordinates.to_cell(end)?,
        })
    }

    pub(super) fn selected_row(
        &self,
        buffer: &Buffer,
        row: usize,
        left: usize,
        width: u16,
    ) -> Result<Option<Range<u16>>, Error> {
        let Some(rectangle) = &self.rectangle else {
            return Ok(self.row(row, left, width));
        };
        let columns = rectangle.columns(buffer, row)?;
        Ok(columns.and_then(|columns| clipped(columns, left, width)))
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

#[derive(Default)]
struct Attributes {
    colors: Option<super::style::Colors>,
    underlined: bool,
    reversed: bool,
}

impl Row {
    pub(super) fn write(
        &self,
        writer: &mut impl Write,
        palette: &crate::config::Palette,
    ) -> io::Result<()> {
        let mut attributes = Attributes::default();
        for glyph in Glyphs::for_text(&self.text).map_err(io::Error::other)? {
            self.write_glyph(
                writer,
                glyph.map_err(io::Error::other)?,
                &mut attributes,
                palette,
            )?;
        }
        underline(writer, false, &mut attributes.underlined)?;
        reverse(writer, false, &mut attributes.reversed)?;
        super::style::reset(writer)
    }

    fn write_glyph(
        &self,
        writer: &mut impl Write,
        glyph: crate::coordinates::Glyph<'_>,
        attributes: &mut Attributes,
        palette: &crate::config::Palette,
    ) -> io::Result<()> {
        let column = glyph.columns.start;
        self.colors(column, palette)
            .write(writer, &mut attributes.colors)?;
        let monochrome = crossterm::style::Colored::ansi_color_disabled_memoized();
        reverse(
            writer,
            monochrome && covers(&self.selection, column),
            &mut attributes.reversed,
        )?;
        underline(
            writer,
            covers(&self.search, column),
            &mut attributes.underlined,
        )?;
        writer.write_all(glyph.text.as_bytes())
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

fn clipped(columns: Range<usize>, left: usize, width: u16) -> Option<Range<u16>> {
    let start = columns.start.saturating_sub(left).min(usize::from(width));
    let end = columns.end.saturating_sub(left).min(usize::from(width));
    (start < end).then_some(start as u16..end as u16)
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
