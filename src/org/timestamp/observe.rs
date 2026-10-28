use super::*;
use crate::coordinates::{Coordinates, LineByte};
use std::ops::Range;

pub(super) fn at(buffer: &Buffer, point: Scalar) -> Result<Target, String> {
    let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
    let byte = coordinates.to_byte(point).map_err(|e| e.to_string())?.0;
    let span = line_span(buffer, point)?;
    let base = coordinates
        .to_byte(Scalar(span.range.start))
        .map_err(|e| e.to_string())?
        .0;
    let text = buffer.slice(&span).map_err(|e| e.to_string())?;
    if text.len_bytes() > 4096 {
        return Err("Org timestamp observation exceeds a 4096-byte logical line".into());
    }
    let text = text.to_string();
    let Some(range) = candidate(&text, byte - base)? else {
        return Target::new(buffer, byte..byte);
    };
    observed(buffer, base, &text, range, byte - base)
}
fn line_span(buffer: &Buffer, point: Scalar) -> Result<crate::buffer::Span, String> {
    let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
    let row = c.to_line(point).map_err(|e| e.to_string())?.row;
    let start = c
        .from_line(LineByte { row, column: 0 })
        .map_err(|e| e.to_string())?;
    let end = if row + 1 < buffer.len_lines() {
        c.from_line(LineByte {
            row: row + 1,
            column: 0,
        })
        .map_err(|e| e.to_string())?
    } else {
        Scalar(buffer.len_chars())
    };
    buffer.span(start.0..end.0).map_err(|e| e.to_string())
}

fn observed(
    buffer: &Buffer,
    base: usize,
    text: &str,
    range: Range<usize>,
    point: usize,
) -> Result<Target, String> {
    let range = connected(text, range)?;
    let mut target = Target::new(buffer, base + range.start..base + range.end)?;
    let first = text[range.clone()]
        .find(['>', ']'])
        .ok_or("Unclosed Org timestamp")?;
    target.selected_end = point > range.start + first + 1;
    target.value = Some(TimestampRange::parse(&text[range])?);
    Ok(target)
}
fn candidate(text: &str, at: usize) -> Result<Option<Range<usize>>, String> {
    for (start, open) in text.match_indices(['<', '[']) {
        if start > at {
            break;
        }
        if !date_prefix(&text[start + 1..]) {
            continue;
        }
        let close = if open == "<" { '>' } else { ']' };
        let end = text[start..].find(close).map(|n| start + n + 1);
        if end.is_none() {
            return Err("Unclosed Org timestamp at point".into());
        }
        let end = end.expect("checked closing delimiter");
        if at <= end {
            return Ok(Some(start..end));
        }
    }
    Ok(None)
}
fn date_prefix(text: &str) -> bool {
    text.as_bytes()
        .get(..5)
        .is_some_and(|s| s[..4].iter().all(u8::is_ascii_digit) && s[4] == b'-')
}
fn connected(text: &str, range: Range<usize>) -> Result<Range<usize>, String> {
    if text[range.end..].starts_with("--") {
        let start = range.end + 2;
        let tail = &text[start..];
        if !tail.starts_with(['<', '[']) {
            return Err("Malformed Org date range".into());
        }
        let end = tail.find(['>', ']']).ok_or("Unclosed Org date range")?;
        return Ok(range.start..start + end + 1);
    }
    let head = &text[..range.start];
    if let Some(head) = head.strip_suffix("--")
        && let Some(start) = head.rfind(['<', '['])
    {
        return Ok(start..range.end);
    }
    Ok(range)
}
