use std::borrow::Cow;

use ropey::RopeSlice;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

use super::{Byte, Error};

type Query<T> = fn(&mut GraphemeCursor, &str, usize) -> Result<T, GraphemeIncomplete>;

pub(super) struct Cursor<'a> {
    text: RopeSlice<'a>,
    cursor: GraphemeCursor,
    chunk: Cow<'a, str>,
    start: usize,
}

/// The segmentation cursor carries RI/ZWJ context across cooperative batches.
/// Only chunk coordinates are retained; source text remains in the buffer.
pub(super) struct Checkpoint {
    cursor: GraphemeCursor,
    chunk: std::ops::Range<usize>,
}

impl<'a> Cursor<'a> {
    pub(super) fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            cursor: self.cursor.clone(),
            chunk: self.start..self.start + self.chunk.len(),
        }
    }

    pub(super) fn resume(text: RopeSlice<'a>, at: Checkpoint) -> Result<Self, Error> {
        let chunk = text
            .get_byte_slice(at.chunk.clone())
            .ok_or(Error::Segmentation)?;
        Ok(Self {
            text,
            cursor: at.cursor,
            chunk: chunk
                .as_str()
                .map_or_else(|| Cow::Owned(chunk.to_string()), Cow::Borrowed),
            start: at.chunk.start,
        })
    }

    pub(super) fn new(text: RopeSlice<'a>, byte: usize) -> Result<Self, Error> {
        text.get_byte_slice(byte..byte)
            .ok_or(Error::Byte(Byte(byte)))?;
        let (chunk, start, _, _) = text
            .try_chunk_at_byte(byte)
            .map_err(|_| Error::Byte(Byte(byte)))?;
        Ok(Self {
            text,
            cursor: GraphemeCursor::new(byte, text.len_bytes(), true),
            chunk: Cow::Borrowed(chunk),
            start,
        })
    }

    pub(super) fn next(&mut self) -> Result<Option<usize>, Error> {
        self.query(GraphemeCursor::next_boundary)
    }

    pub(super) fn previous(&mut self) -> Result<Option<usize>, Error> {
        self.query(GraphemeCursor::prev_boundary)
    }

    pub(super) fn boundary(&mut self) -> Result<bool, Error> {
        self.query(GraphemeCursor::is_boundary)
    }

    fn query<T>(&mut self, query: Query<T>) -> Result<T, Error> {
        loop {
            match query(&mut self.cursor, &self.chunk, self.start) {
                Ok(value) => return Ok(value),
                Err(need) => self.supply(need)?,
            }
        }
    }

    fn supply(&mut self, need: GraphemeIncomplete) -> Result<(), Error> {
        match need {
            GraphemeIncomplete::PreContext(end) => self.context(end),
            GraphemeIncomplete::PrevChunk => {
                self.select(self.start.checked_sub(1).ok_or(Error::Segmentation)?)
            }
            GraphemeIncomplete::NextChunk => self.select(self.start + self.chunk.len()),
            GraphemeIncomplete::InvalidOffset => Err(Error::Segmentation),
        }
    }

    fn select(&mut self, byte: usize) -> Result<(), Error> {
        let (chunk, start, _, _) = self
            .text
            .try_chunk_at_byte(byte)
            .map_err(|_| Error::Segmentation)?;
        if start == self.start {
            return Err(Error::Segmentation);
        }
        let (chunk, start) = if start > self.start {
            self.overlap(chunk, start)?
        } else {
            (Cow::Borrowed(chunk), start)
        };
        self.chunk = chunk;
        self.start = start;
        Ok(())
    }

    fn overlap(&self, chunk: &'a str, start: usize) -> Result<(Cow<'a, str>, usize), Error> {
        // A repeated cursor must not recount cached regional-indicator context
        // at an exact chunk start. One preceding scalar keeps that context valid.
        let scalar = self
            .text
            .try_byte_to_char(start)
            .map_err(|_| Error::Segmentation)?;
        let begin = self
            .text
            .try_char_to_byte(scalar.saturating_sub(1))
            .map_err(|_| Error::Segmentation)?;
        let text = self
            .text
            .get_byte_slice(begin..start + chunk.len())
            .ok_or(Error::Segmentation)?;
        Ok((Cow::Owned(text.to_string()), begin))
    }

    fn context(&mut self, end: usize) -> Result<(), Error> {
        let byte = end.checked_sub(1).ok_or(Error::Segmentation)?;
        let (chunk, start, _, _) = self
            .text
            .try_chunk_at_byte(byte)
            .map_err(|_| Error::Segmentation)?;
        let prefix = chunk.get(..end - start).ok_or(Error::Segmentation)?;
        self.cursor.provide_context(prefix, start);
        Ok(())
    }
}
