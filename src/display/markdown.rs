use crate::buffer::Buffer;
use crate::coordinates::{Byte, Coordinates};
use crate::view::{Projection, View};

use super::Error;

/// Substitute only admitted ASCII marker cells. Keeping their source width
/// lets cursor, pointer, selection and wrapping share the canonical projection.
pub(super) fn project(
    buffer: &Buffer,
    view: &View,
    projection: Projection,
    width: u16,
    text: &mut String,
) -> Result<(), Error> {
    if !view.pretty_markdown() {
        return Ok(());
    }
    heading(buffer, projection, width, text)?;
    for item in buffer.markdown_items(projection.source.row) {
        if let crate::markdown::Marker::Bullet(source) = item.form {
            substitute(
                buffer,
                projection,
                width,
                text,
                &Decoration {
                    bytes: item.marker.clone(),
                    source,
                    glyph: ["•", "◦", "▪"][(item.depth - 1) % 3],
                },
            )?;
        }
    }
    Ok(())
}

fn heading(
    buffer: &Buffer,
    projection: Projection,
    width: u16,
    text: &mut String,
) -> Result<(), Error> {
    let Some((level, bytes)) = buffer.markdown_heading_prefix(projection.source.row) else {
        return Ok(());
    };
    substitute(
        buffer,
        projection,
        width,
        text,
        &Decoration {
            bytes,
            source: b'#',
            glyph: ["◉", "○", "✸", "✿", "◆", "▶"][usize::from(level) - 1],
        },
    )
}

struct Decoration {
    bytes: std::ops::Range<usize>,
    source: u8,
    glyph: &'static str,
}

fn substitute(
    buffer: &Buffer,
    projection: Projection,
    width: u16,
    text: &mut String,
    marker: &Decoration,
) -> Result<(), Error> {
    let coordinates = Coordinates::new(buffer)?;
    let start = coordinates
        .to_cell(coordinates.from_byte(Byte(marker.bytes.start))?)?
        .column;
    let end = start + marker.bytes.len();
    let left = projection.source.column;
    let visible = start.max(left)..end.min(left.saturating_add(usize::from(width)));
    if visible.is_empty() {
        return Ok(());
    }
    let offset = visible.start - left + usize::from(projection.margin);
    let byte = cell_byte(text, offset);
    let cells = visible.len();
    if !text
        .get(byte..byte + cells)
        .is_some_and(|part| part.bytes().all(|b| b == marker.source))
    {
        return Ok(());
    }
    let mut replacement = " ".repeat(cells);
    if visible.end == end {
        replacement.pop();
        replacement.push_str(marker.glyph);
    }
    text.replace_range(byte..byte + cells, &replacement);
    Ok(())
}

fn cell_byte(text: &str, target: usize) -> usize {
    use unicode_width::UnicodeWidthChar;
    let mut column = 0;
    for (byte, character) in text.char_indices() {
        if column == target {
            return byte;
        }
        column += character.width().unwrap_or(0);
    }
    text.len()
}
