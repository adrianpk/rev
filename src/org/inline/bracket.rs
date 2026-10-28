use super::read::{Read, Stop};
use super::{InlineKind, LinkKind, Raw};

pub(super) fn recognize(read: &mut Read<'_>, start: usize) -> Result<Option<Raw>, Stop> {
    if read.starts(start, b"[[")? {
        return link(read, start);
    }
    if read.starts(start, b"[fn:")? {
        return footnote(read, start);
    }
    Ok(None)
}

fn link(read: &mut Read<'_>, start: usize) -> Result<Option<Raw>, Stop> {
    let Some(close) = path_end(read, start + 2)? else {
        return Ok(None);
    };
    if close == start + 2 {
        return Ok(None);
    }
    let mut node = Raw::new(
        InlineKind::Link(LinkKind::Bracket),
        start..close + 2,
        start + 2..close,
    );
    if read.byte(close + 1)? == Some(b']') {
        return Ok(Some(node));
    }
    if read.byte(close + 1)? != Some(b'[') {
        return Ok(None);
    }
    let Some(end) = description_end(read, close + 2)? else {
        return Ok(None);
    };
    if end == close + 2 {
        return Ok(None);
    }
    node.source.end = end + 2;
    node.detail = Some(close + 2..end);
    Ok(Some(node))
}

fn path_end(read: &mut Read<'_>, mut position: usize) -> Result<Option<usize>, Stop> {
    while let Some(byte) = read.byte(position)? {
        match byte {
            b'\\' => position += 2,
            b'[' => return Ok(None),
            b']' => return Ok(Some(position)),
            _ => position += 1,
        }
    }
    Ok(None)
}

fn description_end(read: &mut Read<'_>, mut position: usize) -> Result<Option<usize>, Stop> {
    if position < read.failed_until[6] {
        return Ok(None);
    }
    while let Some(byte) = read.byte(position)? {
        if byte == b']' && read.byte(position + 1)? == Some(b']') {
            return Ok(Some(position));
        }
        position += 1;
    }
    read.failed_until[6] = read.end;
    Ok(None)
}

fn footnote(read: &mut Read<'_>, start: usize) -> Result<Option<Raw>, Stop> {
    let position = label_end(read, start + 4)?;
    if read.byte(position)? == Some(b']') && position > start + 4 {
        return Ok(Some(Raw::new(
            InlineKind::FootnoteReference,
            start..position + 1,
            start + 4..position,
        )));
    }
    if read.byte(position)? != Some(b':') {
        return Ok(None);
    }
    let Some(end) = balanced(read, start)? else {
        return Ok(None);
    };
    let mut node = Raw::new(
        InlineKind::FootnoteInline,
        start..end + 1,
        start + 4..position,
    );
    node.detail = Some(position + 1..end);
    Ok(Some(node))
}

fn label_end(read: &mut Read<'_>, start: usize) -> Result<usize, Stop> {
    let mut position = start;
    while let Some(character) = read.character(position)? {
        if !character.is_alphanumeric() && !"_-".contains(character) {
            break;
        }
        position += character.len_utf8();
    }
    Ok(position)
}

fn balanced(read: &mut Read<'_>, start: usize) -> Result<Option<usize>, Stop> {
    let mut depth = 1;
    let mut position = start + 1;
    while let Some(byte) = read.byte(position)? {
        if byte == b'[' {
            depth += 1;
        }
        if read.depth + depth > read.max_depth {
            return Err(Stop(crate::org::Coverage::Incomplete(
                crate::org::Limit::SourceDepth,
            )));
        }
        if byte == b']' {
            depth -= 1;
        }
        if depth == 0 {
            return Ok(Some(position));
        }
        position += 1;
    }
    Ok(None)
}
