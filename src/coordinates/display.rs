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
    pub(crate) fn into_owned(self) -> Glyph<'static> {
        Glyph {
            source: self.source,
            columns: self.columns,
            text: Cow::Owned(self.text.into_owned()),
        }
    }

    fn new(
        text: RopeSlice<'a>,
        source: Range<Scalar>,
        column: usize,
        tab_width: usize,
    ) -> Result<Self, Error> {
        let text = text
            .as_str()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(text.to_string()));
        let (text, width) = presentation(text, column, tab_width);
        let end = column.checked_add(width).ok_or(Error::Overflow)?;
        Ok(Self {
            source,
            columns: column..end,
            text,
        })
    }
}

/// Projects a line incrementally with bounded chunk context and per-cluster text.
pub struct Glyphs<'a> {
    text: RopeSlice<'a>,
    context_start: Scalar,
    cursor: Cursor<'a>,
    byte: usize,
    point: Scalar,
    column: usize,
    tab_width: usize,
    done: bool,
}

pub(crate) struct Checkpoint {
    context_start: Scalar,
    cursor: super::cursor::Checkpoint,
    byte: usize,
    point: Scalar,
    column: usize,
    done: bool,
}

impl<'a> Glyphs<'a> {
    pub(crate) fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            context_start: self.context_start,
            cursor: self.cursor.checkpoint(),
            byte: self.byte,
            point: self.point,
            column: self.column,
            done: self.done,
        }
    }

    pub(super) fn resume(
        text: RopeSlice<'a>,
        start: Scalar,
        tab_width: usize,
        at: Checkpoint,
    ) -> Result<Self, Error> {
        let offset = at
            .context_start
            .0
            .checked_sub(start.0)
            .ok_or(Error::Segmentation)?;
        let text = text.get_slice(offset..).ok_or(Error::Segmentation)?;
        Ok(Self {
            text,
            context_start: at.context_start,
            cursor: Cursor::resume(text, at.cursor)?,
            byte: at.byte,
            point: at.point,
            column: at.column,
            tab_width,
            done: at.done,
        })
    }

    pub(crate) fn for_text(text: &'a str) -> Result<Self, Error> {
        Self::new(
            RopeSlice::from(text),
            Scalar(0),
            crate::config::Editor::default().tab_width,
        )
    }

    /// Measure a text fragment with the same glyph/tab policy as the source
    /// canvas, starting at its actual terminal column. No source identity is invented.
    pub(crate) fn for_columns(
        text: &'a str,
        column: usize,
        tab_width: usize,
    ) -> Result<Self, Error> {
        let mut glyphs = Self::new(RopeSlice::from(text), Scalar(0), tab_width)?;
        glyphs.column = column;
        Ok(glyphs)
    }

    pub(super) fn new(text: RopeSlice<'a>, start: Scalar, tab_width: usize) -> Result<Self, Error> {
        Ok(Self {
            text,
            context_start: start,
            cursor: Cursor::new(text, 0)?,
            byte: 0,
            point: start,
            column: 0,
            tab_width,
            done: false,
        })
    }

    pub(super) fn at(
        text: RopeSlice<'a>,
        start: Scalar,
        tab_width: usize,
        point: Scalar,
        column: usize,
    ) -> Result<Self, Error> {
        let local = point.0.checked_sub(start.0).ok_or(Error::Scalar(point))?;
        // The caller supplies an observed extended-grapheme boundary. UAX #29
        // section 6.3 permits finding the next boundary from that known break.
        // Starting the suffix here avoids rescanning an arbitrarily long RI
        // prefix. Tabs still use the observed absolute logical column.
        let text = text.get_slice(local..).ok_or(Error::Scalar(point))?;
        Ok(Self {
            text,
            context_start: point,
            cursor: Cursor::new(text, 0)?,
            byte: 0,
            point,
            column,
            tab_width,
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
        let glyph = Glyph::new(text, source.clone(), self.column, self.tab_width)?;
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

fn presentation(text: Cow<'_, str>, column: usize, tab_width: usize) -> (Cow<'_, str>, usize) {
    if text == "\t" {
        let width = tab_width - column % tab_width;
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

#[cfg(test)]
mod tests {
    use super::*;

    // Known-boundary suffix restarts must preserve every source span, absolute
    // tab column and glyph. Include RI parity and context-sensitive EGC rules
    // across rope chunks; arbitrary scalar offsets are not admitted here.
    #[test]
    fn observed_boundaries() {
        for source in [
            format!("{}🇨", "🇦🇧".repeat(600)),
            "\u{600}a e\u{301}\t👩‍💻 क्\u{200d}ष क्ष 각 \r\n".repeat(40),
        ] {
            let rope = ropey::Rope::from_str(&source);
            let expected = Glyphs::new(rope.slice(..), Scalar(13), 4)
                .unwrap()
                .map(|g| {
                    let g = g.unwrap();
                    (g.source, g.columns, g.text.into_owned())
                })
                .collect::<Vec<_>>();
            for (index, (span, columns, _)) in expected.iter().enumerate() {
                let actual = Glyphs::at(rope.slice(..), Scalar(13), 4, span.start, columns.start)
                    .unwrap()
                    .map(|g| {
                        let g = g.unwrap();
                        (g.source, g.columns, g.text.into_owned())
                    })
                    .collect::<Vec<_>>();
                assert_eq!(actual, expected[index..], "{index}");
            }
        }
    }
}
