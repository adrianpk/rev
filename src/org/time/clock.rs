//! Active clock values retain their captured zone and instant. Minute endpoints
//! use the source precision; duration subtracts instants across offset changes.
use super::{Duration, OperationTime, ZonedOperation};

#[derive(Clone, Debug)]
pub struct ClockInterval {
    start: ZonedOperation,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ClockRecord {
    pub source: String,
    pub duration: Duration,
}
impl ClockInterval {
    pub fn new(start: ZonedOperation) -> Self {
        Self { start }
    }
    pub fn open_source(&self) -> String {
        format!("CLOCK: {}", self.start.now().format_timestamp(false))
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
