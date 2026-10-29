use super::*;
use crate::org::time::{CivilTime, Date, Timestamp};
const DAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
pub(super) fn timestamp(now: CivilDateTime, kind: char) -> Result<String, String> {
    let minute = CivilTime::new(now.time.hour(), now.time.minute(), 0)?;
    let clock = matches!(kind, 'T' | 'U').then_some(minute);
    Ok(Timestamp::new(now.date, clock, matches!(kind, 't' | 'T'))?.format())
}
pub(super) fn format(now: CivilDateTime, text: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let code = chars.next().ok_or("Trailing % in capture time format")?;
            result.push_str(&match code {
                '%' => "%".into(),
                'n' => "\n".into(),
                't' => "\t".into(),
                _ => field(now, code)?,
            });
        } else {
            result.push(c);
        }
    }
    Ok(result)
}
fn field(now: CivilDateTime, code: char) -> Result<String, String> {
    let date = now.date;
    let value = match code {
        'Y' => format!("{:04}", date.year()),
        'm' => format!("{:02}", date.month()),
        'd' => format!("{:02}", date.day()),
        'e' => format!("{:2}", date.day()),
        'F' => date.to_string(),
        _ => return clock(now, code),
    };
    Ok(value)
}
fn clock(now: CivilDateTime, code: char) -> Result<String, String> {
    let time = now.time;
    let value = match code {
        'H' => format!("{:02}", time.hour()),
        'M' => format!("{:02}", time.minute()),
        'S' => format!("{:02}", time.second()),

        'R' => format!("{:02}:{:02}", time.hour(), time.minute()),
        'T' => format!(
            "{:02}:{:02}:{:02}",
            time.hour(),
            time.minute(),
            time.second()
        ),
        _ => return named(now.date, code),
    };
    Ok(value)
}
fn named(date: Date, code: char) -> Result<String, String> {
    let day = DAYS[usize::from(date.weekday() - 1)];
    let month = MONTHS[usize::from(date.month() - 1)];
    let value = match code {
        'a' => day[..3].into(),
        'A' => day.into(),
        'b' | 'h' => month[..3].into(),
        'B' => month.into(),
        'u' => date.weekday().to_string(),
        'w' => (date.weekday() % 7).to_string(),
        'j' => format!("{:03}", date.days_since(Date::new(date.year(), 1, 1)?) + 1),
        _ => return Err(format!("Unsupported capture time format %{code}")),
    };
    Ok(value)
}

/// Admit the native formatting subset before any prompt or destination read.
pub(super) fn validate(text: &str) -> Result<(), String> {
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let code = chars.next().ok_or("Trailing % in capture time format")?;
            if !"YmdeHMSFRTaAbhBuwj%nt".contains(code) {
                return Err(format!("Unsupported capture time format %{code}"));
            }
        }
    }
    Ok(())
}
