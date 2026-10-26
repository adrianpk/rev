use std::collections::VecDeque;
use std::ops::Range;

use ropey::RopeSlice;

use super::case::TABLE;

#[derive(Clone, Copy)]
pub(super) struct Token {
    pub character: char,
    position: usize,
    first: bool,
    last: bool,
}

/// At most three folded scalars per source scalar, with no per-scalar allocation.
pub(super) fn letters(
    value: char,
    sensitive: bool,
    backward: bool,
) -> impl DoubleEndedIterator<Item = char> + ExactSizeIterator {
    let mut values = [value; 3];
    let mut length = 1;
    if !sensitive {
        length = mapped(value, &mut values);
    }
    if backward {
        values[..length].reverse();
    }
    values.into_iter().take(length)
}

fn mapped(value: char, values: &mut [char; 3]) -> usize {
    if value.is_ascii() {
        values[0] = value.to_ascii_lowercase();
        return 1;
    }
    if let Ok(index) = TABLE.binary_search_by_key(&value, |entry| entry.0) {
        let mapping = TABLE[index].1;
        values[..mapping.len()].copy_from_slice(mapping);
        return mapping.len();
    }
    1
}

pub(super) fn stream(
    source: RopeSlice<'_>,
    offset: usize,
    backward: bool,
    sensitive: bool,
) -> impl Iterator<Item = Token> + '_ {
    let mut characters = source.chars();
    let mut position = offset;
    if backward {
        characters = source.chars_at(source.len_chars()).reversed();
        position += source.len_chars();
    }
    characters.flat_map(move |character| {
        if backward {
            position -= 1;
        }
        let origin = position;
        if !backward {
            position += 1;
        }
        tokens(character, origin, backward, sensitive)
    })
}

fn tokens(
    character: char,
    position: usize,
    backward: bool,
    sensitive: bool,
) -> impl Iterator<Item = Token> {
    let letters = letters(character, sensitive, backward);
    let length = letters.len();
    letters.enumerate().map(move |(index, character)| Token {
        character,
        position,
        first: index == 0,
        last: index + 1 == length,
    })
}

/// A match must consume whole source scalar expansions in either direction.
/// The ring retains only the latest query-length window of original positions.
pub(super) fn range(origins: &VecDeque<Token>) -> Option<Range<usize>> {
    let first = origins.front()?;
    let last = origins.back()?;
    (first.first && last.last)
        .then_some(first.position.min(last.position)..first.position.max(last.position) + 1)
}
