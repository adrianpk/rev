//! Date entry is bounded and pure: `now` and the original value are captured by
//! the prompt owner, never reread from ambient time while the user is typing.
mod clock;
mod date;
mod relative;
mod week;
use super::{CivilDateTime, Timestamp, TimestampRange};

impl TimestampRange {
    /// Interpret the reference date-prompt forms against one operation clock.
    /// Empty input retains the complete original source spelling.
    pub fn enter(&self, input: &str, now: CivilDateTime) -> Result<Self, String> {
        let limit = if input.trim_start().starts_with(['<', '[']) {
            514
        } else {
            256
        };
        if input.len() > limit || input.chars().any(char::is_control) {
            return Err(format!(
                "Org date input requires at most {limit} bytes without control characters"
            ));
        }
        let input = input.trim();
        if input.is_empty() {
            return Ok(self.clone());
        }
        if input.starts_with(['<', '[']) {
            return Self::parse(input);
        }
        self.enter_range(input, now)
    }
    fn enter_range(&self, input: &str, now: CivilDateTime) -> Result<Self, String> {
        let (first, second) = split_input(input)?;
        let start = enter_one(first, &self.start, now)?;
        let end = match second {
            Some(text) => Some(enter_one(text, &start, now)?),
            None => self.end.clone(),
        };
        let result = Self { start, end };
        result.check()?;
        Ok(result)
    }
}
fn enter_one(input: &str, original: &Timestamp, now: CivilDateTime) -> Result<Timestamp, String> {
    let mut result = original.clone();
    let (date_text, clock) = clock::extract(input.trim());
    apply_date(&mut result, date_text, now)?;
    if let Some(clock) = clock {
        result.set_time(Some(clock.start))?;
        result.set_end_time(clock.end)?;
    }
    Ok(result)
}

fn split_input(input: &str) -> Result<(&str, Option<&str>), String> {
    if input.starts_with(['+', '-']) || clock::read(input).is_ok() {
        return Ok((input, None));
    }
    let pair = input
        .split_once("--")
        .map_or((input, None), |(a, b)| (a, Some(b)));
    if pair.1.is_some_and(|s| s.trim().is_empty()) {
        return Err("Org date range requires an end date".into());
    }
    Ok(pair)
}
fn apply_date(result: &mut Timestamp, text: &str, now: CivilDateTime) -> Result<(), String> {
    if let Some(change) = relative::read(
        text,
        CivilDateTime {
            date: result.date(),
            time: result.time().unwrap_or(now.time),
        },
        now,
    )? {
        result.set_date(change.0.date);
        if change.1 {
            result.set_time(Some(change.0.time))?;
            result.set_end_time(None)?;
        }
    } else if !text.is_empty() {
        result.set_date(date::read(text, result.date(), now.date)?);
    }
    Ok(())
}
