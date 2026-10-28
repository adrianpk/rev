use std::ops::Range;

use crate::buffer::Buffer;
use crate::config::{Palette, Rgb};
use crate::coordinates::{Coordinates, Glyph};
use crate::syntax::{Capture, Style};
use crate::view::View;

use super::Error;

/// Terminal-cell spans are derived from source byte captures, never substituted
/// for them. Projection clips complete graphemes and preserves blank edge cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paint {
    pub columns: Range<u16>,
    pub style: Style,
}

pub(super) fn row(
    buffer: &Buffer,
    view: &View,
    row: usize,
    width: u16,
) -> Result<Vec<Paint>, Error> {
    if row >= buffer.len_lines() || !colored(buffer) {
        return Ok(Vec::new());
    }
    let coordinates = Coordinates::new(buffer)?;
    let visible = visible(&coordinates, view, row, width)?;
    let (Some(first), Some(last)) = (visible.first(), visible.last()) else {
        return Ok(Vec::new());
    };
    let bytes = coordinates.to_byte(first.source.start)?.0..coordinates.to_byte(last.source.end)?.0;
    let captures = if buffer.is_org() {
        buffer.org_captures(bytes)
    } else {
        buffer.syntax().captures(buffer, bytes)
    };
    paint(&coordinates, &visible, &captures, view.left)
}

fn visible<'a>(
    coordinates: &Coordinates<'a>,
    view: &View,
    row: usize,
    width: u16,
) -> Result<Vec<Glyph<'a>>, Error> {
    let right = view.left.saturating_add(usize::from(width));
    let visible = coordinates
        .glyphs(row)?
        .take_while(|glyph| match glyph {
            Ok(glyph) => glyph.columns.start < right,
            Err(_) => true,
        })
        .filter(|glyph| match glyph {
            Ok(glyph) => glyph.columns.start >= view.left && glyph.columns.end <= right,
            Err(_) => true,
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(visible)
}

fn paint(
    coordinates: &Coordinates<'_>,
    glyphs: &[Glyph<'_>],
    captures: &[Capture],
    left: usize,
) -> Result<Vec<Paint>, Error> {
    let mut result: Vec<Paint> = Vec::new();
    for glyph in glyphs {
        let bytes =
            coordinates.to_byte(glyph.source.start)?.0..coordinates.to_byte(glyph.source.end)?.0;
        if let Some(capture) = captures
            .iter()
            .rev()
            .find(|capture| capture.bytes.start < bytes.end && capture.bytes.end > bytes.start)
        {
            let columns = (glyph.columns.start - left) as u16..(glyph.columns.end - left) as u16;
            append(
                &mut result,
                Paint {
                    columns,
                    style: capture.style,
                },
            );
        }
    }
    Ok(result)
}

fn append(result: &mut Vec<Paint>, paint: Paint) {
    if let Some(last) = result.last_mut()
        && last.style == paint.style
        && last.columns.end == paint.columns.start
    {
        last.columns.end = paint.columns.end;
    } else {
        result.push(paint);
    }
}

// Capture classification stays syntax-owned; terminal colors resolve only at
// presentation against the one frame palette, including grouped captures.
pub(super) fn foreground(style: Style, palette: &Palette) -> Rgb {
    match style {
        Style::Org(role) => role.color(&palette.org),
        Style::Keyword => palette.syntax.keyword,
        Style::String => palette.syntax.string,
        Style::Comment => palette.syntax.comment,
        Style::Type => palette.syntax.r#type,
        Style::Function => palette.syntax.function,
        Style::Number => palette.syntax.number,
        Style::Punctuation => palette.syntax.punctuation,
    }
}

impl super::Row {
    pub(super) fn style(&self, column: usize) -> Option<Style> {
        self.syntax
            .iter()
            .find(|paint| {
                usize::from(paint.columns.start) <= column
                    && column < usize::from(paint.columns.end)
            })
            .map(|paint| paint.style)
    }
}

fn colored(buffer: &Buffer) -> bool {
    buffer.has_org_colors() || buffer.mode() == "Rust"
}
