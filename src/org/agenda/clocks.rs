use crate::buffer::Span;
use crate::org::time::{Date, Duration, RecordedClock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockScope {
    Task,
    Subtree,
    File,
}
/// An explicit civil date window [start, end), resolved once in the operation's zone.
#[derive(Clone, Copy, Debug)]
pub struct ClockQuery {
    pub scope: ClockScope,
    pub window: Option<crate::org::time::CivilInterval>,
}
impl ClockQuery {
    pub fn parse(source: &str) -> Result<Self, String> {
        if source.len() > 256 {
            return Err("Clock query exceeds 256 UTF-8 bytes".into());
        }
        let words: Vec<_> = source.split_whitespace().take(4).collect();
        let scope = match words.first().copied().unwrap_or("subtree") {
            "task" => ClockScope::Task,
            "subtree" => ClockScope::Subtree,
            "file" => ClockScope::File,
            _ => return Err("Clock scope requires task, subtree or file".into()),
        };
        let window = dates(&words)?
            .map(|(start, end)| {
                let time = crate::org::time::CivilTime::new(0, 0, 0)?;
                crate::org::time::CivilInterval::new(
                    crate::org::time::CivilDateTime { date: start, time },
                    crate::org::time::CivilDateTime { date: end, time },
                )
            })
            .transpose()?;
        Ok(Self { scope, window })
    }
}
pub(super) fn date(source: &str) -> Result<Date, String> {
    let stamp = crate::org::time::Timestamp::parse(&format!("[{source}]"))?;
    if source.len() != 10 {
        return Err("Clock date requires YYYY-MM-DD".into());
    }
    Ok(stamp.date())
}
/// A row retains exact source revision and heading identity for later refresh.
#[derive(Clone, Debug)]
pub struct ClockRow {
    pub source: Span,
    pub title: String,
    pub level: usize,
    pub parent: Option<usize>,
    pub minutes: u64,
    pub effort: Option<Duration>,
    pub open: usize,
    pub unresolved: usize,
}
#[derive(Clone, Debug)]
pub struct ClockSummary {
    pub source: Span,
    pub rows: Vec<ClockRow>,
    pub minutes: u64,
    pub effort: Option<Duration>,
    pub open: usize,
    pub unresolved: usize,
    pub overlaps: usize,
}
impl ClockSummary {
    pub fn feedback(&self) -> Result<String, String> {
        let actual = Duration::from_minutes(self.minutes as f64)?.format_clock(false)?;
        let effort = self
            .effort
            .map(|v| v.format_clock(false))
            .transpose()?
            .unwrap_or("absent".into());
        Ok(format!(
            "Org clock: actual {actual}; effort {effort}; open {}; unresolved {}; overlaps {}",
            self.open, self.unresolved, self.overlaps
        ))
    }
}
pub(crate) fn overlaps(records: &mut [RecordedClock]) -> usize {
    records.sort_by_key(|v| v.start);
    let mut until = i64::MIN;
    let mut count = 0;
    for record in records {
        count += usize::from(record.start < until);
        until = until.max(record.end);
    }
    count
}

fn dates(words: &[&str]) -> Result<Option<(Date, Date)>, String> {
    let dates = match words.len() {
        0 | 1 => None,
        3 => Some((date(words[1])?, date(words[2])?)),
        _ => return Err("Clock query: scope [start-date end-date], end exclusive".into()),
    };
    if dates.is_some_and(|(start, end)| start >= end) {
        return Err("Clock date window requires start before end".into());
    }
    Ok(dates)
}
