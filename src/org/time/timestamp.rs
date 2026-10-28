//! Bounded Org timestamp data. Observing or formatting a value never authorizes
//! a source edit; callers retain canonical revision-qualified source ranges.
mod adjust;
mod parse;
pub use adjust::TimestampField;
mod suffix;
use super::{CivilDateTime, CivilTime, Date};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    date: Date,
    time: Option<CivilTime>,
    end_time: Option<CivilTime>,
    active: bool,
    spelling: Spelling,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Spelling {
    date: Date,
    time: Option<CivilTime>,
    end_time: Option<CivilTime>,
    day: String,
    clock: String,
    suffix: String,
    normalize: bool,
}
/// Two dates use the compatible `--` separator. A same-day clock range belongs
/// to one timestamp; an end clock before its start crosses midnight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampRange {
    pub start: Timestamp,
    pub end: Option<Timestamp>,
}
impl Timestamp {
    pub fn new(date: Date, time: Option<CivilTime>, active: bool) -> Result<Self, String> {
        checked_time(time)?;
        Ok(Self {
            date,
            time,
            end_time: None,
            active,
            spelling: Spelling {
                date,
                time: None,
                end_time: None,
                day: format!(" {}", weekday(date)),
                clock: String::new(),
                suffix: String::new(),
                normalize: false,
            },
        })
    }
    pub fn parse(text: &str) -> Result<Self, String> {
        let result = parse::timestamp(text)?;
        result.end_civil()?;
        Ok(result)
    }
    pub(super) fn suffix(&self) -> &str {
        &self.spelling.suffix
    }
    pub(super) fn remove_first_delay(&mut self) {
        let text = &self.spelling.suffix;
        if let Some(start) = text.find("--") {
            let tail = &text[start..];
            let end = start + tail.find(char::is_whitespace).unwrap_or(tail.len());
            let start = text[..start].trim_end().len();
            self.spelling.suffix.replace_range(start..end, "");
        }
    }
    pub fn date(&self) -> Date {
        self.date
    }
    pub fn time(&self) -> Option<CivilTime> {
        self.time
    }
    pub fn end_time(&self) -> Option<CivilTime> {
        self.end_time
    }
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn set_date(&mut self, date: Date) {
        self.date = date;
    }
    pub fn set_time(&mut self, time: Option<CivilTime>) -> Result<(), String> {
        checked_time(time)?;
        self.time = time;
        if time.is_none() {
            self.end_time = None;
        }
        Ok(())
    }
    pub fn set_end_time(&mut self, end: Option<CivilTime>) -> Result<(), String> {
        checked_time(end)?;
        if end.is_some() && self.time.is_none() {
            return Err("Org end time requires a start time".into());
        }
        self.end_time = end;
        Ok(())
    }
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    pub fn normalize_weekday(&mut self) {
        self.spelling.normalize = true;
    }
    pub fn civil(&self) -> Option<CivilDateTime> {
        self.time.map(|time| CivilDateTime {
            date: self.date,
            time,
        })
    }
    pub fn end_civil(&self) -> Result<Option<CivilDateTime>, String> {
        let Some(end) = self.end_time else {
            return Ok(None);
        };
        let date = if self.time.is_some_and(|start| end < start) {
            self.date.add_days(1)?
        } else {
            self.date
        };
        Ok(Some(CivilDateTime { date, time: end }))
    }
    pub fn format(&self) -> String {
        let (open, close) = if self.active { ('<', '>') } else { ('[', ']') };
        let day = if self.date == self.spelling.date && !self.spelling.normalize {
            self.spelling.day.clone()
        } else {
            format!(" {}", weekday(self.date))
        };
        let clock = self.clock_spelling();
        let separator = if !clock.is_empty() && !day.ends_with(char::is_whitespace) {
            " "
        } else {
            ""
        };
        let mut body = format!("{}{day}{separator}{clock}", self.date);
        if needs_separator(&body, &self.spelling.suffix) {
            body.push(' ');
        }
        body.push_str(&self.spelling.suffix);
        format!("{open}{body}{close}")
    }
    fn clock_spelling(&self) -> String {
        if self.time == self.spelling.time && self.end_time == self.spelling.end_time {
            return self.spelling.clock.clone();
        }
        let Some(time) = self.time else {
            return String::new();
        };
        let (old_start, old_end) = self
            .spelling
            .clock
            .split_once('-')
            .unwrap_or((self.spelling.clock.as_str(), ""));
        let start = clock_part(time, self.spelling.time, old_start);
        match self.end_time {
            Some(end) => format!(
                "{start}-{}",
                clock_part(end, self.spelling.end_time, old_end)
            ),
            None => start,
        }
    }
}
impl TimestampRange {
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() > 514 {
            return Err("Org timestamp range exceeds 514 UTF-8 bytes".into());
        }
        let end = parse::closing(text)? + 1;
        let start = Timestamp::parse(&text[..end])?;
        let second = match text.get(end..) {
            Some("") => None,
            Some(tail) => Some(Timestamp::parse(
                tail.strip_prefix("--")
                    .ok_or("Invalid Org date range separator")?,
            )?),
            None => return Err("Invalid Org timestamp range".into()),
        };
        let range = Self { start, end: second };
        range.check()?;
        Ok(range)
    }
    pub fn check(&self) -> Result<(), String> {
        for stamp in std::iter::once(&self.start).chain(self.end.iter()) {
            stamp.end_civil()?;
        }
        if let Some(end) = &self.end
            && (end.active != self.start.active || end.date < self.start.date)
        {
            return Err("Org date range requires ordered dates and matching delimiters".into());
        }
        Ok(())
    }
    pub fn format(&self) -> String {
        match &self.end {
            Some(end) => format!("{}--{}", self.start.format(), end.format()),
            None => self.start.format(),
        }
    }
}
pub(super) fn weekday(date: Date) -> &'static str {
    ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][usize::from(date.weekday() - 1)]
}
fn checked_time(time: Option<CivilTime>) -> Result<(), String> {
    if time.is_some_and(|t| t.second() != 0) {
        return Err("Org timestamps use whole minutes".into());
    }
    Ok(())
}

fn needs_separator(left: &str, right: &str) -> bool {
    !right.is_empty()
        && !right.starts_with(char::is_whitespace)
        && !left.ends_with(char::is_whitespace)
}

fn clock_part(time: CivilTime, previous: Option<CivilTime>, source: &str) -> String {
    let (old_hour, old_minute) = source.split_once(':').unwrap_or(("", ""));
    let hour = if previous.is_some_and(|old| old.hour() == time.hour()) {
        old_hour.to_owned()
    } else {
        format!("{:02}", time.hour())
    };
    let minute = if previous.is_some_and(|old| old.minute() == time.minute()) {
        old_minute.to_owned()
    } else {
        format!("{:02}", time.minute())
    };
    format!("{hour}:{minute}")
}
