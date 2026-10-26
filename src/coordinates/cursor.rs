use ropey::RopeSlice;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

use super::{Byte, Error};

type Query<T> = fn(&mut GraphemeCursor, &str, usize) -> Result<T, GraphemeIncomplete>;

pub(super) struct Cursor<'a> {
    text: RopeSlice<'a>,
    cursor: GraphemeCursor,
    chunk: &'a str,
    start: usize,
}

impl<'a> Cursor<'a> {
    pub(super) fn new(text: RopeSlice<'a>, byte: usize) -> Result<Self, Error> {
        text.get_byte_slice(byte..byte)
            .ok_or(Error::Byte(Byte(byte)))?;
        let (chunk, start, _, _) = text
            .try_chunk_at_byte(byte)
            .map_err(|_| Error::Byte(Byte(byte)))?;
        Ok(Self {
            text,
            cursor: GraphemeCursor::new(byte, text.len_bytes(), true),
            chunk,
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
            match query(&mut self.cursor, self.chunk, self.start) {
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
        self.chunk = chunk;
        self.start = start;
        Ok(())
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
