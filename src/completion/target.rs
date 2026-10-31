use crate::buffer::{Buffer, Span, edit::Point};
use crate::coordinates::Coordinates;

pub(super) fn at(
    buffer: &Buffer,
    point: Point,
) -> Result<(Span, String), crate::buffer::edit::Error> {
    buffer.slice(&Span {
        stamp: point.stamp,
        range: point.position.0..point.position.0,
    })?;
    Coordinates::new(buffer)?.to_cell(point.position)?;
    let mut start = point.position.0;
    let mut end = start;
    while start > 0 && identifier(character(buffer, start - 1)?) {
        start -= 1;
        limit(point.position.0 - start)?;
    }
    while end < buffer.len_chars() && identifier(character(buffer, end)?) {
        end += 1;
        limit(end - start)?;
    }
    let source = buffer.span(start..end)?;
    let prefix = buffer
        .slice(&buffer.span(start..point.position.0)?)?
        .to_string();
    Ok((source, prefix))
}

fn character(buffer: &Buffer, position: usize) -> Result<char, crate::buffer::Error> {
    Ok(buffer.slice(&buffer.span(position..position + 1)?)?.char(0))
}

fn limit(length: usize) -> Result<(), crate::buffer::edit::Error> {
    if length > super::MAX_LABEL {
        return Err(crate::buffer::edit::Error::Capacity);
    }
    Ok(())
}

fn identifier(ch: char) -> bool {
    ch == '_'
        || ch.is_alphanumeric()
        || (!ch.is_ascii() && unicode_width::UnicodeWidthChar::width(ch) == Some(0))
}
