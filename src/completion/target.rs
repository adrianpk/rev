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
    let start = start(buffer, point.position.0)?;
    let end = end(buffer, point.position.0)?;
    limit(end - start)?;
    let source = buffer.span(start..end)?;
    let prefix = buffer
        .slice(&buffer.span(start..point.position.0)?)?
        .to_string();
    Ok((source, prefix))
}

fn start(buffer: &Buffer, point: usize) -> Result<usize, crate::buffer::edit::Error> {
    let mut start = point;
    while start > 0 && identifier(character(buffer, start - 1)?, buffer.completion_grammar()) {
        start -= 1;
        limit(point - start)?;
    }
    Ok(raw_start(buffer, start)?)
}

fn end(buffer: &Buffer, point: usize) -> Result<usize, crate::buffer::edit::Error> {
    let mut end = point;
    while end < buffer.len_chars()
        && identifier(character(buffer, end)?, buffer.completion_grammar())
    {
        end += 1;
        limit(end - point)?;
    }
    Ok(end)
}

fn raw_start(buffer: &Buffer, start: usize) -> Result<usize, crate::buffer::Error> {
    if start >= 2 && character(buffer, start - 1)? == '#' && character(buffer, start - 2)? == 'r' {
        return Ok(start - 2);
    }
    Ok(start)
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

fn identifier(ch: char, grammar: crate::config::Grammar) -> bool {
    ch == '_'
        || ch.is_alphanumeric()
        || (!ch.is_ascii() && unicode_width::UnicodeWidthChar::width(ch) == Some(0))
        || extra(ch, grammar)
}

fn extra(ch: char, grammar: crate::config::Grammar) -> bool {
    use crate::config::Grammar;
    match grammar {
        Grammar::Html => ch == '-' || ch == ':',
        Grammar::Css => ch == '-',
        Grammar::Javascript | Grammar::Typescript | Grammar::Tsx => ch == '$' || ch == '#',
        _ => false,
    }
}
