use crate::coordinates::{Error, Glyphs};

/// Edge fragments of a grapheme become spaces; following glyphs keep their
/// absolute cell alignment. Tabs already consist of their expanded spaces.
pub(super) fn line(glyphs: Glyphs<'_>, left: usize, width: u16) -> Result<String, Error> {
    let mut text = String::new();
    let mut column = 0;
    let right = left.saturating_add(usize::from(width));
    for glyph in glyphs {
        let glyph = glyph?;
        if glyph.columns.start >= right {
            break;
        }
        if glyph.columns.end <= left {
            continue;
        }
        let start = glyph.columns.start.max(left) - left;
        let end = glyph.columns.end.min(right) - left;
        pad(&mut text, start - column);
        if glyph.columns.start >= left && glyph.columns.end <= right {
            text.push_str(&glyph.text);
        } else {
            pad(&mut text, end - start);
        }
        column = end;
    }
    pad(&mut text, usize::from(width) - column);
    Ok(text)
}

fn pad(text: &mut String, cells: usize) {
    text.extend(std::iter::repeat_n(' ', cells));
}
