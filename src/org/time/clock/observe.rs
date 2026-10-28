use super::super::{CivilDateTime, Resolution, Timestamp, Zone};

/// Source-derived UTC minute bounds. No duration field or file provenance can
/// establish an instant when a zone-less source timestamp is ambiguous.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordedClock {
    pub start: i64,
    pub end: i64,
}
impl RecordedClock {
    /// Observe one compatible CLOCK line; None denotes a valid open interval.
    /// Malformed, gap and fold records remain unresolved rather than guessed.
    pub fn observe(source: &str, zone: &Zone) -> Result<Option<Self>, String> {
        size(source)?;
        let source = body(source)?;
        let (start, tail) = timestamp(source)?;
        if tail.trim().is_empty() {
            return Ok(None);
        }
        closed(start, tail, zone).map(Some)
    }
    pub fn clipped(self, range: Option<Self>) -> Option<Self> {
        let range = range.unwrap_or(self);
        let start = self.start.max(range.start);
        let end = self.end.min(range.end);
        (start < end).then_some(Self { start, end })
    }
    pub fn minutes(self) -> u64 {
        (self.end - self.start) as u64
    }
}
fn timestamp(source: &str) -> Result<(CivilDateTime, &str), String> {
    let end = source
        .find(']')
        .ok_or("CLOCK requires an inactive timestamp")?;
    let value = Timestamp::parse(&source[..=end])?;
    validate(&value)?;
    Ok((
        value.civil().ok_or("CLOCK requires a time")?,
        &source[end + 1..],
    ))
}
fn validate(value: &Timestamp) -> Result<(), String> {
    if value.active() || value.time().is_none() || value.end_time().is_some() {
        return Err("CLOCK requires one inactive timestamp with a time".into());
    }
    if value.repeater()?.is_some() || value.warning_delay()?.is_some() {
        return Err("CLOCK timestamp cannot repeat or carry a warning".into());
    }
    Ok(())
}
fn instant(value: CivilDateTime, zone: &Zone) -> Result<i64, String> {
    match zone.resolve(value)? {
        Resolution::Unique(value) => Ok(value.unix_seconds.div_euclid(60)),
        Resolution::Ambiguous(_) => Err("CLOCK source has an ambiguous local time".into()),
        Resolution::Nonexistent => Err("CLOCK source has a nonexistent local time".into()),
    }
}
fn duration(tail: &str) -> Result<(), String> {
    let value = tail
        .trim()
        .strip_prefix("=>")
        .ok_or("CLOCK requires recorded duration")?
        .trim();
    let (hours, minutes) = value
        .split_once(':')
        .ok_or("CLOCK duration requires H:MM")?;
    let valid = !hours.is_empty() && hours.bytes().all(|b| b.is_ascii_digit());
    if !valid || minutes.len() != 2 || !minutes.bytes().all(|b| b.is_ascii_digit()) {
        return Err("CLOCK duration requires H:MM".into());
    }
    let minute: u8 = minutes.parse().map_err(|_| "Invalid CLOCK minutes")?;
    let _: u64 = hours.parse().map_err(|_| "CLOCK hours overflow")?;
    if minute >= 60 {
        return Err("Invalid CLOCK minutes".into());
    }
    Ok(())
}

fn size(source: &str) -> Result<(), String> {
    if source.len() > 4096 {
        return Err("Org CLOCK line exceeds 4096 UTF-8 bytes".into());
    }
    Ok(())
}

fn body(source: &str) -> Result<&str, String> {
    Ok(source
        .trim()
        .strip_prefix("CLOCK:")
        .ok_or("Expected CLOCK record")?
        .trim())
}

fn closed(start: CivilDateTime, tail: &str, zone: &Zone) -> Result<RecordedClock, String> {
    let tail = tail
        .trim()
        .strip_prefix("--")
        .ok_or("Invalid CLOCK end separator")?;
    let (end, tail) = timestamp(tail)?;
    duration(tail)?;
    let start = instant(start, zone)?;
    let end = instant(end, zone)?;
    if end < start {
        return Err("CLOCK ends before its start".into());
    }
    Ok(RecordedClock { start, end })
}
