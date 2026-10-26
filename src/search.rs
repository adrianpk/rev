use std::fmt;

use crate::buffer::{self, Buffer, Span};
use crate::coordinates::{self, Scalar};

pub const LIMIT: usize = 4096;

#[derive(Debug)]
pub enum Error {
    Buffer(buffer::Error),
    Coordinates(coordinates::Error),
    Limit,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Buffer(error) => error.fmt(formatter),
            Self::Coordinates(error) => error.fmt(formatter),
            Self::Limit => formatter.write_str("search limit is 4096 characters"),
        }
    }
}

impl std::error::Error for Error {}

impl From<buffer::Error> for Error {
    fn from(error: buffer::Error) -> Self {
        Self::Buffer(error)
    }
}

impl From<coordinates::Error> for Error {
    fn from(error: coordinates::Error) -> Self {
        Self::Coordinates(error)
    }
}

/// A bounded literal query and KMP fallback table. Source text remains borrowed;
/// scans carry prefix state across rope chunks and report original scalar ranges.
pub struct Pattern {
    text: Vec<char>,
    fallback: Vec<usize>,
}

impl Pattern {
    pub fn new(query: &str) -> Result<Self, Error> {
        let text: Vec<_> = query.chars().take(LIMIT + 1).collect();
        if text.len() > LIMIT {
            return Err(Error::Limit);
        }
        let fallback = prefixes(&text);
        Ok(Self { text, fallback })
    }

    /// Finds the first literal match at or after a scalar, without wrapping.
    /// The result qualifies its exact source range with the searched revision.
    pub fn find(&self, buffer: &Buffer, start: Scalar) -> Result<Option<Span>, Error> {
        let source = buffer.span(start.0..buffer.len_chars())?;
        let slice = buffer.slice(&source)?;
        if self.text.is_empty() {
            return Ok(None);
        }
        let mut matched = 0;
        for (index, character) in slice.chunks().flat_map(str::chars).enumerate() {
            matched = self.advance(matched, character);
            if matched == self.text.len() {
                let end = start.0 + index + 1;
                return Ok(Some(buffer.span(end - self.text.len()..end)?));
            }
        }
        Ok(None)
    }

    fn advance(&self, mut matched: usize, character: char) -> usize {
        while matched > 0 && self.text[matched] != character {
            matched = self.fallback[matched - 1];
        }
        if self.text[matched] == character {
            matched += 1;
        }
        matched
    }
}

fn prefixes(text: &[char]) -> Vec<usize> {
    let mut table = vec![0; text.len()];
    for index in 1..text.len() {
        let mut matched = table[index - 1];
        while matched > 0 && text[matched] != text[index] {
            matched = table[matched - 1];
        }
        if text[matched] == text[index] {
            matched += 1;
        }
        table[index] = matched;
    }
    table
}
