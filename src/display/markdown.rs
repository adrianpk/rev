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
    let Some((level, prefix)) = buffer.markdown_heading_prefix(projection.source.row) else {
        return Ok(());
    };
    let coordinates = Coordinates::new(buffer)?;
    let start = coordinates
        .to_cell(coordinates.from_byte(Byte(prefix.start))?)?
        .column;
    let end = start + usize::from(level);
    let left = projection.source.column;
    let visible = start.max(left)..end.min(left.saturating_add(usize::from(width)));
    if visible.is_empty() {
        return Ok(());
    }
    let offset = visible.start - left + usize::from(projection.margin);
    // The prefix is ASCII; preceding visible graphemes can have different byte
    // and cell widths (for example a tab in a container prefix).
    let byte = cell_byte(text, offset);
    let cells = visible.len();
    if !text
        .get(byte..byte + cells)
        .is_some_and(|part| part.bytes().all(|b| b == b'#'))
    {
        return Ok(());
    }
    let mut replacement = " ".repeat(cells);
    if visible.end == end {
        replacement.pop();
        replacement.push_str(["◉", "○", "✸", "✿", "◆", "▶"][usize::from(level) - 1]);
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
