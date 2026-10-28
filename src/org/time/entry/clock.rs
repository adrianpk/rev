use super::super::CivilTime;

pub(super) struct Clock {
    pub(super) start: CivilTime,
    pub(super) end: Option<CivilTime>,
}
/// A clock must consume the input tail. Candidate work is bounded by the date
/// entry's 256-byte limit; UTF-8 positions always come from char boundaries.
pub(super) fn extract(text: &str) -> (&str, Option<Clock>) {
    if text.starts_with(['+', '-']) && !text.contains(char::is_whitespace) {
        return (text, None);
    }
    for (at, character) in text.char_indices() {
        if character.is_ascii_digit()
            && !text
                .as_bytes()
                .get(at.wrapping_sub(1))
                .is_some_and(u8::is_ascii_digit)
            && let Ok(clock) = read(&text[at..])
        {
            return (text[..at].trim(), Some(clock));
        }
    }
    (text, None)
}
pub(super) fn read(text: &str) -> Result<Clock, String> {
    let text = text.trim();
    if let Some((start, duration)) = text.split_once('+') {
        let start = single(start)?;
        let minutes = duration_minutes(duration)?;
        let end = minutes_of_day(start) + minutes;
        return Ok(Clock {
            start,
            end: Some(from_minutes(end % 1440)?),
        });
    }
    let pair = text.split_once("--").or_else(|| text.split_once('-'));
    let (start, end) = pair.map_or((text, None), |(a, b)| (a, Some(b)));
    Ok(Clock {
        start: single(start)?,
        end: end.map(single).transpose()?,
    })
}
fn single(text: &str) -> Result<CivilTime, String> {
    let text = text.trim();
    let lower = text.to_ascii_lowercase();
    let (clock, period) = period(&lower);
    let (hour, minute) = if period.is_some() && !clock.contains([':', 'h']) {
        (number(clock)?, 0)
    } else {
        parts(clock)?
    };
    let hour = adjusted_hour(hour, period)?;
    CivilTime::new(hour, minute, 0)
}
fn period(text: &str) -> (&str, Option<bool>) {
    if let Some(clock) = text.strip_suffix("am") {
        return (clock, Some(false));
    }
    if let Some(clock) = text.strip_suffix("pm") {
        return (clock, Some(true));
    }
    (text, None)
}
fn parts(text: &str) -> Result<(u8, u8), String> {
    let (hour, minute) = text
        .split_once([':', 'h'])
        .ok_or("Org time requires : or h, or am/pm")?;
    Ok((
        number(hour)?,
        if minute.is_empty() {
            0
        } else {
            number(minute)?
        },
    ))
}
fn adjusted_hour(hour: u8, period: Option<bool>) -> Result<u8, String> {
    let Some(pm) = period else {
        return Ok(hour);
    };
    if !(1..=12).contains(&hour) {
        return Err("Org 12-hour clock requires 1..12".into());
    }
    Ok(hour % 12 + if pm { 12 } else { 0 })
}
fn number(text: &str) -> Result<u8, String> {
    if text.is_empty() || text.len() > 2 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Invalid Org clock number".into());
    }
    text.parse().map_err(|_| "Invalid Org clock number".into())
}
fn duration_minutes(text: &str) -> Result<u16, String> {
    let (hour, minute) = text.split_once(':').unwrap_or((text, "0"));
    let hour = u16::from(number(hour)?);
    let minute = u16::from(number(minute)?);
    let total = hour * 60 + minute;
    if minute > 59 || total == 0 || total >= 1440 {
        return Err("Org clock range duration requires 1..1439 minutes".into());
    }
    Ok(total)
}
fn minutes_of_day(time: CivilTime) -> u16 {
    u16::from(time.hour()) * 60 + u16::from(time.minute())
}
fn from_minutes(minutes: u16) -> Result<CivilTime, String> {
    CivilTime::new((minutes / 60) as u8, (minutes % 60) as u8, 0)
}
