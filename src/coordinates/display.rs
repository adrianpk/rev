use std::borrow::Cow;
use std::ops::Range;

use ropey::RopeSlice;
use unicode_width::UnicodeWidthStr;

use super::{Error, Scalar, cursor::Cursor};

/// Safe presentation for one grapheme, with explicit source and display spans.
pub struct Glyph<'a> {
    pub source: Range<Scalar>,
    pub columns: Range<usize>,
    pub text: Cow<'a, str>,
}

impl<'a> Glyph<'a> {
    fn new(text: RopeSlice<'a>, source: Range<Scalar>, column: usize) -> Result<Self, Error> {
        let text = text
            .as_str()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(text.to_string()));
        let (text, width) = presentation(text, column);
        let end = column.checked_add(width).ok_or(Error::Overflow)?;
        Ok(Self {
            source,
            columns: column..end,
            text,
        })
    }
}

/// Projects a line incrementally; only a cluster crossing chunks is joined.
pub struct Glyphs<'a> {
    text: RopeSlice<'a>,
    cursor: Cursor<'a>,
    byte: usize,
    point: Scalar,
    column: usize,
    done: bool,
}

impl<'a> Glyphs<'a> {
    pub(super) fn new(text: RopeSlice<'a>, start: Scalar) -> Result<Self, Error> {
        Ok(Self {
            text,
            cursor: Cursor::new(text, 0)?,
            byte: 0,
            point: start,
            column: 0,
            done: false,
        })
    }

    fn advance(&mut self) -> Result<Option<Glyph<'a>>, Error> {
        let Some(end) = self.cursor.next()? else {
            return Ok(None);
        };
        let text = self
            .text
            .get_byte_slice(self.byte..end)
            .ok_or(Error::Segmentation)?;
        let source = self.point..Scalar(self.point.0 + text.len_chars());
        let glyph = Glyph::new(text, source.clone(), self.column)?;
        self.byte = end;
        self.point = source.end;
        self.column = glyph.columns.end;
        Ok(Some(glyph))
    }
}

impl<'a> Iterator for Glyphs<'a> {
    type Item = Result<Glyph<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.advance() {
            Ok(Some(glyph)) => Some(Ok(glyph)),
            Ok(None) => {
                self.done = true;
                None
            }
            Err(error) => {
                self.done = true;
                Some(Err(error))
            }
        }
    }
}

fn presentation(text: Cow<'_, str>, column: usize) -> (Cow<'_, str>, usize) {
    if text == "\t" {
        let width = 4 - column % 4;
        return (Cow::Owned(" ".repeat(width)), width);
    }
    if let Some(control) = text.chars().next().filter(|ch| ch.is_control()) {
        let visible = control_text(control);
        let width = visible.len();
        return (Cow::Owned(visible), width);
    }
    let width = text.width();
    if width == 0 {
        return (Cow::Owned(format!("◌{text}")), 1);
    }
    (text, width)
}

fn control_text(control: char) -> String {
    if control <= '\u{1f}' {
        return format!("^{}", char::from(control as u8 + 64));
    }
    if control == '\u{7f}' {
        return "^?".into();
    }
    format!("<{:02X}>", control as u32)
}
