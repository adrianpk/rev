use super::super::{CalendarInterval, CivilDateTime, CivilTime, Date};
struct Relative<'a> {
    base: CivilDateTime,
    count: i64,
    suffix: &'a str,
    backward: bool,
}

/// The single sign selects the captured current date; the double sign selects
/// the prompt's original date. Hours retain their selected base's clock time.
pub(super) fn read(
    text: &str,
    default: CivilDateTime,
    now: CivilDateTime,
) -> Result<Option<(CivilDateTime, bool)>, String> {
    let lower = text.to_ascii_lowercase();
    let Some(change) = parts(&lower, default, now)? else {
        return Ok(None);
    };
    let zero = CivilTime::new(0, 0, 0)?;
    if let Some(day) = super::date::weekday(change.suffix) {
        let date = occurrence(&change, day)?;
        return Ok(Some((CivilDateTime { date, time: zero }, false)));
    }
    let interval = interval(change.suffix, change.count)?;
    let time = if change.suffix == "h" {
        change.base.time
    } else {
        zero
    };
    let time = CivilTime::new(time.hour(), time.minute(), 0)?;
    let result = CivilDateTime {
        date: change.base.date,
        time,
    }
    .shift(interval)?;
    Ok(Some((result, change.suffix == "h")))
}
fn parts<'a>(
    text: &'a str,
    default: CivilDateTime,
    today: CivilDateTime,
) -> Result<Option<Relative<'a>>, String> {
    if !text.starts_with(['+', '-']) {
        return Ok(None);
    }
    if text.contains(char::is_whitespace) {
        return Err("Unexpected text after Org relative date".into());
    }
    let sign = text.as_bytes()[0];
    let same = text.as_bytes().get(1) == Some(&sign);
    let rest = &text[1 + usize::from(same)..];
    let end = rest.bytes().take_while(u8::is_ascii_digit).count();
    let count: i64 = if end == 0 {
        1
    } else {
        rest[..end]
            .parse()
            .map_err(|_| "Org relative date count overflows")?
    };
    Ok(Some(Relative {
        base: if same { default } else { today },
        count: if sign == b'-' { -count } else { count },
        suffix: &rest[end..],
        backward: sign == b'-',
    }))
}
fn interval(suffix: &str, count: i64) -> Result<CalendarInterval, String> {
    match suffix {
        "" | "d" => Ok(CalendarInterval::Days(count)),
        "w" => Ok(CalendarInterval::Weeks(count)),
        "m" => Ok(CalendarInterval::Months(count)),
        "y" => Ok(CalendarInterval::Years(count)),
        "h" => Ok(CalendarInterval::Hours(count)),
        _ => Err("Unknown Org relative date unit".into()),
    }
}
fn occurrence(change: &Relative<'_>, weekday: u8) -> Result<Date, String> {
    let delta = (i64::from(weekday) - i64::from(change.base.date.weekday())).rem_euclid(7);
    let forward = if delta == 0 { 7 } else { delta };
    let edge = if change.backward {
        if forward == 7 { -7 } else { forward - 7 }
    } else {
        forward
    };
    let weeks = if change.backward {
        (change.count + 1).min(0)
    } else {
        change.count.saturating_sub(1).max(0)
    };
    let days = weeks
        .checked_mul(7)
        .and_then(|n| n.checked_add(edge))
        .ok_or("Org relative weekday overflows")?;
    change.base.date.add_days(days)
}
