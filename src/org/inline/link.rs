use super::read::{Read, Stop};
use super::{InlineContext, InlineKind, LinkKind, Raw};

pub(super) fn scheme(read: &mut Read<'_>, range: std::ops::Range<usize>) -> Result<bool, Stop> {
    for name in [
        "id",
        "attachment",
        "file+sys",
        "file+emacs",
        "shell",
        "news",
        "mailto",
        "https",
        "http",
        "ftp",
        "shortdoc",
        "help",
        "file",
        "elisp",
    ] {
        if read.equals_ascii_case(range.clone(), name)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn angle(
    read: &mut Read<'_>,
    start: usize,
    context: InlineContext,
) -> Result<Option<Raw>, Stop> {
    if start < read.failed_until[7] {
        return Ok(None);
    }
    if read.starts(start, b"<<")? {
        return target(read, start, context);
    }
    if context != InlineContext::Standard {
        return Ok(None);
    }
    let Some(colon) = colon(read, start + 1, read.end)? else {
        return Ok(None);
    };
    if !scheme(read, start + 1..colon)? {
        return Ok(None);
    }
    let Some(end) = read.find(colon + 1, b'>')? else {
        read.failed_until[7] = read.end;
        return Ok(None);
    };
    Ok(Some(Raw::new(
        InlineKind::Link(LinkKind::Angle),
        start..end + 1,
        start + 1..end,
    )))
}

fn target(read: &mut Read<'_>, start: usize, context: InlineContext) -> Result<Option<Raw>, Stop> {
    if context != InlineContext::Standard {
        return Ok(None);
    }
    let radio = read.starts(start, b"<<<")?;
    target_body(read, start, radio)
}

fn target_body(read: &mut Read<'_>, start: usize, radio: bool) -> Result<Option<Raw>, Stop> {
    let width = if radio { 3 } else { 2 };
    let Some(end) = target_end(read, start + width)? else {
        return Ok(None);
    };
    if !valid_target(read, start + width, end)? {
        return Ok(None);
    }
    let closing = if radio {
        b">>>".as_slice()
    } else {
        b">>".as_slice()
    };
    if !read.starts(end, closing)? {
        return Ok(None);
    }
    let kind = if radio {
        InlineKind::RadioTarget
    } else {
        InlineKind::Target
    };
    Ok(Some(Raw::new(kind, start..end + width, start + width..end)))
}

fn target_end(read: &mut Read<'_>, mut position: usize) -> Result<Option<usize>, Stop> {
    while let Some(byte) = read.byte(position)? {
        match byte {
            b'>' => return Ok(Some(position)),
            b'<' | b'\n' => return Ok(None),
            _ => position += 1,
        }
    }
    read.failed_until[7] = read.end;
    Ok(None)
}

fn valid_target(read: &mut Read<'_>, start: usize, end: usize) -> Result<bool, Stop> {
    if start == end {
        return Ok(false);
    }
    if read.character(start)?.is_none_or(char::is_whitespace) {
        return Ok(false);
    }
    if read.previous(end)?.is_none_or(char::is_whitespace) {
        return Ok(false);
    }
    let mut position = start;
    while position < end {
        if matches!(read.byte(position)?, Some(b'<' | b'\n')) {
            return Ok(false);
        }
        position += 1;
    }
    Ok(true)
}

fn colon(read: &mut Read<'_>, start: usize, end: usize) -> Result<Option<usize>, Stop> {
    let mut position = start;
    while position < end {
        match read.byte(position)? {
            Some(b':') => return Ok(Some(position)),
            Some(byte) if byte.is_ascii_alphanumeric() || byte == b'+' => position += 1,
            _ => return Ok(None),
        }
    }
    Ok(None)
}

pub(super) fn plain(read: &mut Read<'_>, start: usize) -> Result<Option<Raw>, Stop> {
    if !plain_start(read, start)? {
        return Ok(None);
    }
    let Some(colon) = colon(read, start, read.end)? else {
        return Ok(None);
    };
    if !scheme(read, start..colon)? {
        return Ok(None);
    }
    let end = path(read, colon + 1)?;
    if end == colon + 1 {
        return Ok(None);
    }
    Ok(Some(Raw::new(
        InlineKind::Link(LinkKind::Plain),
        start..end,
        start..end,
    )))
}

fn plain_start(read: &mut Read<'_>, start: usize) -> Result<bool, Stop> {
    if !read
        .byte(start)?
        .is_some_and(|byte| byte.is_ascii_alphabetic())
    {
        return Ok(false);
    }
    if read
        .previous(start)?
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
    {
        return Ok(false);
    }
    Ok(true)
}

fn path(read: &mut Read<'_>, start: usize) -> Result<usize, Stop> {
    let mut position = start;
    let mut end = start;
    let mut depth = 0;
    while let Some(byte) = read.byte(position)? {
        if !path_byte(byte, &mut depth) {
            break;
        }
        position += 1;
        if depth == 0 && (!byte.is_ascii_punctuation() || byte == b'/' || byte == b')') {
            end = position;
        }
    }
    Ok(end)
}

fn path_byte(byte: u8, depth: &mut usize) -> bool {
    if byte.is_ascii_whitespace() || b"[]<>\"".contains(&byte) {
        return false;
    }
    match byte {
        b'(' => *depth += 1,
        b')' if *depth == 0 => return false,
        b')' => *depth -= 1,
        _ => {}
    }
    *depth <= 2
}
