use super::{CivilDateTime, CivilTime, Timestamp};
use crate::org::time::CalendarInterval;

/// A selected field in a formatted timestamp. Repeat/warning interval source
/// does not become a date field merely because it contains digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimestampField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    EndHour,
    EndMinute,
    Active,
}
impl TimestampField {
    fn interval(self, up: bool) -> Option<CalendarInterval> {
        let n = if up { 1 } else { -1 };
        match self {
            Self::Year => Some(CalendarInterval::Years(n)),
            Self::Month => Some(CalendarInterval::Months(n)),
            Self::Day => Some(CalendarInterval::Days(n)),
            Self::Hour | Self::EndHour => Some(CalendarInterval::Hours(n)),
            Self::Minute | Self::EndMinute => Some(CalendarInterval::Minutes(n)),
            Self::Active => None,
        }
    }
    fn calendar(self) -> bool {
        matches!(self, Self::Year | Self::Month | Self::Day)
    }
    fn end(self) -> bool {
        matches!(self, Self::EndHour | Self::EndMinute)
    }
}
impl Timestamp {
    /// Point is a UTF-8 byte boundary in this value's formatted source. Date
    /// punctuation belongs to its preceding numeric field; weekday selects day.
    pub fn field_at(&self, point: usize) -> Result<TimestampField, String> {
        let text = self.format();
        if point > text.len() || !text.is_char_boundary(point) {
            return Err("Invalid Org timestamp field boundary".into());
        }
        if point == text.len() {
            return Ok(TimestampField::Day);
        }
        if point == 0 || point + 1 == text.len() {
            return Ok(TimestampField::Active);
        }
        match point {
            1..=5 => return Ok(TimestampField::Year),
            6..=8 => return Ok(TimestampField::Month),
            9..=10 => return Ok(TimestampField::Day),
            _ => (),
        }
        self.content_field(&text, point)
    }
    fn content_field(&self, text: &str, point: usize) -> Result<TimestampField, String> {
        let value = Self::parse(text)?;
        let clock = 11 + value.spelling.day.len();
        if point < clock {
            return Ok(TimestampField::Day);
        }
        let relative = point - clock;
        if relative < value.spelling.clock.len() {
            return clock_field(&value.spelling.clock, relative);
        }
        Err("Org repeat/warning interval adjustment is unavailable here".into())
    }
    /// Adjust one field atomically. Moving the first clock moves its end by the
    /// same civil amount; editing the end clock changes only the range length.
    pub fn step(&mut self, field: TimestampField, up: bool) -> Result<(), String> {
        let mut candidate = self.clone();
        match field.interval(up) {
            Some(shift) => candidate.adjust(field, shift)?,
            None => candidate.active = !candidate.active,
        }
        candidate.end_civil()?;
        *self = candidate;
        Ok(())
    }
    fn adjust(&mut self, field: TimestampField, shift: CalendarInterval) -> Result<(), String> {
        if field.calendar() {
            let value = CivilDateTime {
                date: self.date,
                time: CivilTime::new(0, 0, 0)?,
            }
            .shift(shift)?;
            self.date = value.date;
            return Ok(());
        }
        if field.end() {
            let end = self
                .end_civil()?
                .ok_or("Org timestamp has no end clock")?
                .shift(shift)?;
            self.end_time = Some(end.time);
            return Ok(());
        }
        self.shift_clock(shift)
    }
    fn shift_clock(&mut self, shift: CalendarInterval) -> Result<(), String> {
        let next = self
            .civil()
            .ok_or("Org timestamp has no clock")?
            .shift(shift)?;
        let end = self.end_civil()?.map(|end| end.shift(shift)).transpose()?;
        self.date = next.date;
        self.time = Some(next.time);
        self.end_time = end.map(|end| end.time);
        Ok(())
    }
}
fn clock_field(clock: &str, point: usize) -> Result<TimestampField, String> {
    let end = clock
        .find('-')
        .filter(|index| point > *index)
        .map(|index| index + 1);
    let (clock, point) = end.map_or((clock, point), |at| (&clock[at..], point - at));
    let colon = clock.find(':').ok_or("Org clock has no minute field")?;
    match (end.is_some(), point <= colon) {
        (false, true) => Ok(TimestampField::Hour),
        (false, false) => Ok(TimestampField::Minute),
        (true, true) => Ok(TimestampField::EndHour),
        (true, false) => Ok(TimestampField::EndMinute),
    }
}
