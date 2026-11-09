use std::ops::Range;

use crate::buffer::{Buffer, Span};
use crate::coordinates::{Coordinates, Scalar};

use super::{Direction, Error, Pattern, prefixes};

/// One bounded continuation policy over the canonical literal matcher.
#[derive(Clone, Copy)]
pub struct Request {
    pub anchor: Scalar,
    pub direction: Direction,
    pub count: usize,
    pub whole_word: bool,
}

impl Request {
    pub fn new(anchor: Scalar, direction: Direction) -> Self {
        Self {
            anchor,
            direction,
            count: 1,
            whole_word: false,
        }
    }

    pub(super) fn accept(&mut self, buffer: &Buffer, range: &Range<usize>) -> Result<bool, Error> {
        if self.whole_word && !word_edges(buffer, range)? {
            return Ok(false);
        }
        self.count = self.count.saturating_sub(1);
        Ok(self.count == 0)
    }
}

impl Pattern {
    /// Resolve a count in one rope scan, without wrapping or partial movement.
    pub fn occurrence(&self, buffer: &Buffer, request: Request) -> Result<Option<Span>, Error> {
        if request.direction == Direction::Forward {
            return self.scan(
                buffer,
                &buffer.span(request.anchor.0..buffer.len_chars())?,
                request,
            );
        }
        let text: Vec<_> = self.text.iter().rev().copied().collect();
        let reversed = Self {
            fallback: prefixes(&text),
            text,
            sensitive: self.sensitive,
        };
        reversed.scan(buffer, &buffer.span(0..request.anchor.0)?, request)
    }
}

fn word_edges(buffer: &Buffer, range: &Range<usize>) -> Result<bool, Error> {
    let coordinates = Coordinates::new(buffer)?;
    if !coordinates.is_boundary(Scalar(range.start))?
        || !coordinates.is_boundary(Scalar(range.end))?
    {
        return Ok(false);
    }
    let source = buffer.slice(&buffer.span(0..buffer.len_chars())?)?;
    let before = range
        .start
        .checked_sub(1)
        .and_then(|at| source.get_char(at));
    let after = source.get_char(range.end);
    Ok(!before.is_some_and(word_character) && !after.is_some_and(word_character))
}

fn word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}
