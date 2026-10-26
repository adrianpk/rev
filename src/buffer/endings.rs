use std::ops::Range;

use ropey::{Rope, RopeSlice};

use super::edit::{Change, Error};

pub(super) fn count(text: RopeSlice<'_>) -> usize {
    let mut previous = '\0';
    text.chars()
        .filter(|character| {
            let pair = previous == '\r' && *character == '\n';
            previous = *character;
            pair
        })
        .count()
}

pub(super) fn update(
    before: &Rope,
    after: &Rope,
    changes: &[Change],
    current: usize,
) -> Result<usize, Error> {
    let removed = affected(before, changes.iter().map(Change::before))?;
    let inserted = affected(after, changes.iter().map(Change::after))?;
    current
        .checked_sub(removed)
        .and_then(|count| count.checked_add(inserted))
        .ok_or(Error::Capacity)
}

fn affected(text: &Rope, ranges: impl Iterator<Item = Range<usize>>) -> Result<usize, Error> {
    let mut total = 0;
    for range in windows(ranges, text.len_chars()) {
        let slice = text.get_slice(range).ok_or(Error::Capacity)?;
        total += count(slice);
    }
    Ok(total)
}

fn windows(ranges: impl Iterator<Item = Range<usize>>, len: usize) -> Vec<Range<usize>> {
    let mut windows: Vec<Range<usize>> = Vec::new();
    for range in ranges {
        // Include both seams, and merge touching neighborhoods so a CRLF pair
        // near adjacent base ranges is counted once in each revision.
        let range = range.start.saturating_sub(1)..range.end.saturating_add(1).min(len);
        if let Some(previous) = windows.last_mut()
            && previous.end >= range.start
        {
            previous.end = previous.end.max(range.end);
        } else {
            windows.push(range);
        }
    }
    windows
}
