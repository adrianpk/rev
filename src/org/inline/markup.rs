use super::read::{Read, Stop};
use super::{InlineKind, Raw};

fn kind(marker: u8) -> InlineKind {
    match marker {
        b'*' => InlineKind::Bold,
        b'/' => InlineKind::Italic,
        b'_' => InlineKind::Underline,
        b'+' => InlineKind::Strike,
        b'=' => InlineKind::Verbatim,
        _ => InlineKind::Code,
    }
}

fn before(character: Option<char>) -> bool {
    character.is_none_or(|c| c.is_whitespace() || "-({'\"".contains(c))
}

fn after(character: Option<char>) -> bool {
    character.is_none_or(|c| c.is_whitespace() || "-.,;:!?'\u{29}}[\"\\".contains(c))
}

pub(super) fn recognize(read: &mut Read<'_>, start: usize) -> Result<Option<Raw>, Stop> {
    if !opens(read, start)? {
        return Ok(None);
    }
    search(read, start)
}

fn opens(read: &mut Read<'_>, start: usize) -> Result<bool, Stop> {
    if !before(read.previous(start)?) {
        return Ok(false);
    }
    Ok(read
        .character(start + 1)?
        .is_some_and(|c| !c.is_whitespace()))
}

fn search(read: &mut Read<'_>, start: usize) -> Result<Option<Raw>, Stop> {
    let marker = read.byte(start)?.expect("admitted markup marker");
    let slot = b"*/_+=~"
        .iter()
        .position(|byte| *byte == marker)
        .expect("known markup marker");
    if start < read.failed_until[slot] {
        return Ok(None);
    }
    let mut position = start + 2;
    let mut lines = 0;
    let mut first_line = None;
    while let Some(byte) = read.byte(position)? {
        if byte == b'\n' {
            first_line.get_or_insert(position);
            lines += 1;
        }
        if lines > 1 {
            // Later openers on the first line share this failed search.
            read.failed_until[slot] = first_line.unwrap_or(position);
            return Ok(None);
        }
        if closes(read, position, marker)? {
            return Ok(Some(Raw::new(
                kind(marker),
                start..position + 1,
                start + 1..position,
            )));
        }
        position += 1;
    }
    read.failed_until[slot] = read.end;
    Ok(None)
}

fn closes(read: &mut Read<'_>, position: usize, marker: u8) -> Result<bool, Stop> {
    if read.byte(position)? != Some(marker) {
        return Ok(false);
    }
    if read.previous(position)?.is_some_and(char::is_whitespace) {
        return Ok(false);
    }
    Ok(after(read.character(position + 1)?))
}
