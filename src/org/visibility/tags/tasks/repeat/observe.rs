use super::*;
use crate::coordinates::{Byte, Scalar};
use crate::org::timestamp::Target;

pub(super) fn timestamps(
    model: &Outline,
    buffer: &Buffer,
    index: usize,
) -> Result<Vec<RepeatSource>, String> {
    let heading = &model.headings[index];
    let range = heading.start..heading.section;
    if range.len() > 262144 {
        return Err("Org repeat observation exceeds a 262144-byte entry".into());
    }
    let text = edit::source(buffer, range.clone())?;
    let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    let mut consumed = range.start;
    for (local, _) in text.match_indices('<') {
        let byte = range.start + local;
        let Some(point) = eligible(model, buffer, (byte, &text[local + 1..]), consumed)? else {
            continue;
        };
        if let Some(value) = observed(buffer, point)? {
            consumed = coordinates
                .to_byte(Scalar(value.source.range.end))
                .map_err(|e| e.to_string())?
                .0;
            result.push(value);
        }
        if result.len() > 64 {
            return Err("Org completion exceeds 64 repeating timestamp ranges".into());
        }
    }
    Ok(result)
}
fn eligible(
    model: &Outline,
    buffer: &Buffer,
    location: (usize, &str),
    consumed: usize,
) -> Result<Option<Scalar>, String> {
    let (byte, tail) = location;
    if byte < consumed || !date_prefix(tail) {
        return Ok(None);
    }
    let point = Coordinates::new(buffer)
        .map_err(|e| e.to_string())?
        .from_byte(Byte(byte))
        .map_err(|e| e.to_string())?;
    Ok(model.timestamp_context(buffer, point)?.then_some(point))
}
fn observed(buffer: &Buffer, point: Scalar) -> Result<Option<RepeatSource>, String> {
    let target = Target::at(buffer, point)?;
    let source = buffer
        .slice(&target.source)
        .map_err(|e| e.to_string())?
        .to_string();
    let value = TimestampRange::parse(&source)?;
    if !repeated(&value)? {
        return Ok(None);
    }
    Ok(Some(RepeatSource {
        source: target.source,
        value,
        habit_schedule: false,
    }))
}
fn repeated(value: &TimestampRange) -> Result<bool, String> {
    if value.start.active() && value.start.repeater()?.is_some() {
        return Ok(true);
    }
    match &value.end {
        Some(end) => Ok(end.active() && end.repeater()?.is_some()),
        None => Ok(false),
    }
}
fn date_prefix(text: &str) -> bool {
    text.as_bytes()
        .get(..5)
        .is_some_and(|s| s[..4].iter().all(u8::is_ascii_digit) && s[4] == b'-')
}
