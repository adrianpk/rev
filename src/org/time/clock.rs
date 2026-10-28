//! Active clock values retain their captured zone and instant. Minute endpoints
//! use the source precision; duration subtracts instants across offset changes.
use super::{Duration, OperationTime, ZonedOperation};

#[derive(Clone, Debug)]
pub struct ClockInterval {
    start: ZonedOperation,
    source: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ClockRecord {
    pub source: String,
    pub duration: Duration,
}
impl ClockInterval {
    pub fn new(start: ZonedOperation) -> Self {
        let source = format!("CLOCK: {}", start.now().format_timestamp(false));
        Self { start, source }
    }
    /// Adopt only an explicitly selected source/instant pair in its observed
    /// zone. Original source spelling remains authoritative through clock-out.
    pub fn resume(source: &str, start: ZonedOperation) -> Result<Self, String> {
        let value = source_clock(source)?;
        if !value
            .civil()
            .is_some_and(|civil| same_minute(civil, start.now()))
        {
            return Err("Org CLOCK source does not identify the selected start instant".into());
        }
        Ok(Self {
            start,
            source: source.trim_end().into(),
        })
    }
    pub fn started(&self) -> OperationTime {
        self.start.instant()
    }
    pub fn open_source(&self) -> String {
        self.source.clone()
    }
    pub fn close(&self, end: OperationTime) -> Result<ClockRecord, String> {
        let start = self.start.instant().unix_seconds();
        let stop = end.unix_seconds();
        if stop < start {
            return Err("Org clock ends before it starts; interval retained".into());
        }
        let minutes = stop
            .div_euclid(60)
            .checked_sub(start.div_euclid(60))
            .ok_or("Org clock duration overflows")?;
        let duration = Duration::from_minutes(minutes as f64)?;
        let end = self.start.zone().civil_at(stop)?.format_timestamp(false);
        Ok(ClockRecord {
            source: format!(
                "{}--{end} => {:>5}",
                self.open_source(),
                duration.format_clock(false)?
            ),
            duration,
        })
    }
}

fn source_clock(source: &str) -> Result<super::Timestamp, String> {
    if source.len() > 4096 {
        return Err("Org CLOCK line exceeds 4096 UTF-8 bytes".into());
    }
    let raw = source
        .trim_start()
        .strip_prefix("CLOCK:")
        .ok_or("Expected Org CLOCK source")?
        .trim();
    let value = super::Timestamp::parse(raw)?;
    if value.active() || value.end_time().is_some() {
        return Err("Org CLOCK requires one inactive start timestamp".into());
    }
    if value.repeater()?.is_some() || value.warning_delay()?.is_some() {
        return Err("Org CLOCK cannot contain recurrence or delay data".into());
    }
    Ok(value)
}

fn same_minute(a: super::CivilDateTime, b: super::CivilDateTime) -> bool {
    a.date == b.date && a.time.hour() == b.time.hour() && a.time.minute() == b.time.minute()
}
