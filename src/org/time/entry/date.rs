use super::super::Date;

pub(super) fn read(text: &str, default: Date, today: Date) -> Result<Date, String> {
    let lower = text.trim().to_ascii_lowercase();
    if matches!(lower.as_str(), "." | "today") {
        return Ok(today);
    }
    if let Some(weekday) = weekday(&lower) {
        return today.add_days({
            let delta = (i64::from(weekday) - i64::from(today.weekday())).rem_euclid(7);
            if delta == 0 { 7 } else { delta }
        });
    }
    if lower.contains('w') {
        return super::week::read(&lower, default);
    }
    if lower.contains('/') {
        return numeric(&lower, today, '/');
    }
    if lower.contains('-') {
        return numeric(&lower, today, '-');
    }
    if lower.bytes().all(|b| b.is_ascii_digit()) {
        return next_day(today, number(&lower)?);
    }
    named(&lower, today)
}
fn numeric(text: &str, today: Date, separator: char) -> Result<Date, String> {
    let parts: Vec<_> = text.split(separator).collect();
    match parts.as_slice() {
        [month, day] => next_date(today, number(month)?, number(day)?),
        [a, b, c] => {
            let (year, month, day) = if separator == '-' {
                (a, b, c)
            } else {
                (c, a, b)
            };
            Date::new(
                year_number(year, today.year())?,
                number(month)?,
                number(day)?,
            )
        }
        _ => Err("Invalid Org date fields".into()),
    }
}
fn named(text: &str, today: Date) -> Result<Date, String> {
    // English month names may touch the day number, as in 22sept or sep15.
    let words: Vec<_> = text
        .split(char::is_whitespace)
        .filter(|s| !s.is_empty())
        .collect();
    if words.len() > 3 {
        return Err("Too many Org date fields".into());
    }
    let expanded = words.join(" ");
    let (start, end) = month_bounds(&expanded)?;
    let month = month(&expanded[start..end]).ok_or("Unknown Org month")?;
    let before = expanded[..start].trim();
    let after: Vec<_> = expanded[end..].split_whitespace().collect();
    let (day, year) = named_parts(before, &after)?;
    match year {
        Some(year) => Date::new(year_number(year, today.year())?, month, number(day)?),
        None => next_date(today, month, number(day)?),
    }
}
fn month_bounds(text: &str) -> Result<(usize, usize), String> {
    let start = text
        .find(char::is_alphabetic)
        .ok_or("Org date requires a month or day")?;
    let count = text[start..]
        .chars()
        .take_while(|c| c.is_alphabetic())
        .map(char::len_utf8)
        .sum::<usize>();
    Ok((start, start + count))
}
fn named_parts<'a>(
    before: &'a str,
    after: &[&'a str],
) -> Result<(&'a str, Option<&'a str>), String> {
    match (before.is_empty(), after) {
        (true, [day]) => Ok((day, None)),
        (true, [day, year]) => Ok((day, Some(year))),
        (false, []) => Ok((before, None)),
        (false, [year]) => Ok((before, Some(year))),
        _ => Err("Org named date requires a day and optional year".into()),
    }
}
fn next_day(default: Date, day: u8) -> Result<Date, String> {
    let next = if day < default.day() {
        Date::new(default.year(), default.month(), 1)?.add_months(1)?
    } else {
        default
    };
    Date::new(next.year(), next.month(), day)
}
fn next_date(default: Date, month: u8, day: u8) -> Result<Date, String> {
    let date = Date::new(default.year(), month, day)?;
    if date < default {
        Date::new(
            default.year().checked_add(1).ok_or("Org year overflows")?,
            month,
            day,
        )
    } else {
        Ok(date)
    }
}
fn year_number(text: &str, current: u16) -> Result<u16, String> {
    if text.is_empty() || text.len() > 4 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Invalid Org year".into());
    }
    let year: u16 = text.parse().map_err(|_| "Invalid Org year")?;
    if text.len() > 2 {
        return Ok(year);
    }
    let century = i32::from(current / 100);
    let offset = i32::from(year) - i32::from(current % 100);
    let century = if offset > 30 {
        century - 1
    } else if offset <= -70 {
        century + 1
    } else {
        century
    };
    u16::try_from(century * 100 + i32::from(year))
        .map_err(|_| "Org short year is outside source range".into())
}
fn number(text: &str) -> Result<u8, String> {
    if text.is_empty() || text.len() > 2 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Invalid Org month or day".into());
    }
    text.parse().map_err(|_| "Invalid Org month or day".into())
}
fn month(text: &str) -> Option<u8> {
    let names = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    if text.len() < 3 {
        return None;
    }
    names
        .iter()
        .position(|name| name.starts_with(text))
        .map(|index| index as u8 + 1)
}
pub(super) fn weekday(text: &str) -> Option<u8> {
    let names = [
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
    ];
    if text.len() < 3 {
        return None;
    }
    names
        .iter()
        .position(|name| name.starts_with(text))
        .map(|index| index as u8 + 1)
}
