use std::ops::Range;

use ropey::RopeSlice;

use super::super::{Error, Scalar, cursor::Cursor};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Class {
    Space,
    Word,
    Punctuation,
}

pub(super) struct Run {
    pub range: Range<Scalar>,
    pub class: Class,
}

/// A single rope-backed segmentation cursor, with one cluster of lookahead.
/// Memory is independent of count and document size.
pub(super) struct Runs<'a> {
    text: RopeSlice<'a>,
    cursor: Cursor<'a>,
    byte: usize,
    backward: bool,
    big: bool,
    pending: Option<Run>,
}

impl<'a> Runs<'a> {
    pub fn new(
        text: RopeSlice<'a>,
        point: Scalar,
        backward: bool,
        big: bool,
    ) -> Result<Self, Error> {
        let byte = text
            .try_char_to_byte(point.0)
            .map_err(|_| Error::Scalar(point))?;
        Ok(Self {
            text,
            cursor: Cursor::new(text, byte)?,
            byte,
            backward,
            big,
            pending: None,
        })
    }

    pub fn cluster(&mut self) -> Result<Option<Run>, Error> {
        let next = if self.backward {
            self.cursor.previous()?
        } else {
            self.cursor.next()?
        };
        let Some(next) = next else { return Ok(None) };
        let bytes = self.byte.min(next)..self.byte.max(next);
        let slice = self
            .text
            .get_byte_slice(bytes.clone())
            .ok_or(Error::Segmentation)?;
        let class = classify(slice, self.big);
        self.byte = next;
        Ok(Some(Run {
            range: Scalar(self.text.byte_to_char(bytes.start))
                ..Scalar(self.text.byte_to_char(bytes.end)),
            class,
        }))
    }

    pub fn next(&mut self) -> Result<Option<Run>, Error> {
        let first = match self.pending.take() {
            Some(run) => Some(run),
            None => self.cluster()?,
        };
        let Some(mut run) = first else {
            return Ok(None);
        };
        while let Some(next) = self.cluster()? {
            if next.class != run.class {
                self.pending = Some(next);
                break;
            }
            run.range.start = run.range.start.min(next.range.start);
            run.range.end = run.range.end.max(next.range.end);
        }
        Ok(Some(run))
    }
}

fn classify(text: RopeSlice<'_>, big: bool) -> Class {
    let first = text
        .chars()
        .next()
        .expect("a segmented cluster is nonempty");
    if first.is_whitespace() {
        return Class::Space;
    }
    if big || text.chars().any(|c| c.is_alphanumeric() || c == '_') {
        Class::Word
    } else {
        Class::Punctuation
    }
}
