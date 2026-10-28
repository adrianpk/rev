use super::{CivilTime, Date, Spelling, Timestamp};

pub(super) fn closing(text: &str) -> Result<usize, String> {
    let close = match text.as_bytes().first() {
        Some(b'<') => '>',
        Some(b'[') => ']',
        _ => return Err("Org timestamp requires <...> or [...]".into()),
    };
    text.find(close)
        .ok_or_else(|| "Unclosed Org timestamp".into())
}
pub(super) fn timestamp(text: &str) -> Result<Timestamp, String> {
    if text.len() > 256 || text.chars().any(char::is_control) {
        return Err("Org timestamp requires at most 256 bytes without control characters".into());
    }
    let end = closing(text)?;
    if end + 1 != text.len() {
        return Err("Trailing Org timestamp data".into());
    }
    let content = &text[1..end];
    let date = date(
        content
            .get(..10)
            .ok_or("Org timestamp requires YYYY-MM-DD")?,
    )?;
    let rest = &content[10..];
    let (day, rest) = day(rest)?;
    let clock = clock(rest)?;
    super::suffix::check(clock.suffix)?;
    Ok(Timestamp {
        date,
        time: clock.start,
        end_time: clock.end,
        active: text.starts_with('<'),
        spelling: Spelling {
            date,
            time: clock.start,
            end_time: clock.end,
            day: day.into(),
            clock: clock.source.into(),
            suffix: clock.suffix.into(),
            normalize: false,
        },
    })
}
pub(crate) fn date(text: &str) -> Result<Date, String> {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || !text.is_ascii() || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err("Org date requires YYYY-MM-DD".into());
    }
    let year = digits(&text[..4])? as u16;
    let month = digits(&text[5..7])? as u8;
    let day = digits(&text[8..])? as u8;
    Date::new(year, month, day)
}
fn digits(text: &str) -> Result<u32, String> {
    if !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Org date/time requires ASCII digits".into());
    }
    text.parse()
        .map_err(|_| "Invalid Org date/time number".into())
}
fn day(rest: &str) -> Result<(&str, &str), String> {
    let trim = rest.trim_start();
    if trim.len() == rest.len() && !rest.is_empty() {
        return Err("Org timestamp fields require whitespace".into());
    }
    let trim_day = if trim.starts_with(char::is_alphabetic) {
        trim
    } else {
        ""
    };
    let count = trim_day
        .chars()
        .take_while(|c| c.is_alphabetic() || *c == '.')
        .map(char::len_utf8)
        .sum::<usize>();
    if count > 32 {
        return Err("Org weekday spelling exceeds 32 bytes".into());
    }
    let end = rest.len() - trim.len() + count;
    let tail = &rest[end..];
    let spaces = tail.len() - tail.trim_start().len();
    if count > 0 && !tail.is_empty() && spaces == 0 {
        return Err("Org weekday requires a field separator".into());
    }
    let at = end + spaces;
    Ok((&rest[..at], &rest[at..]))
}
struct Clock<'a> {
    start: Option<CivilTime>,
    end: Option<CivilTime>,
    source: &'a str,
    suffix: &'a str,
}
fn clock(rest: &str) -> Result<Clock<'_>, String> {
    if !rest.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        return Ok(Clock {
            start: None,
            end: None,
            source: "",
            suffix: rest,
        });
    }
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let word = &rest[..end];
    let (start, finish) = word
        .split_once('-')
        .map_or((word, None), |(a, b)| (a, Some(b)));
    let start_time = time(start)?;
    let end_time = finish.map(time).transpose()?;
    Ok(Clock {
        start: Some(start_time),
        end: end_time,
        source: word,
        suffix: &rest[end..],
    })
}
pub(crate) fn time(text: &str) -> Result<CivilTime, String> {
    let (h, m) = text.split_once(':').ok_or("Org time requires H:MM")?;
    if !(1..=2).contains(&h.len()) || m.len() != 2 {
        return Err("Org time requires H:MM or HH:MM".into());
    }
    CivilTime::new(digits(h)? as u8, digits(m)? as u8, 0)
}
