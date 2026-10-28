use super::{CivilDateTime, Duration};
use std::time::{SystemTime, UNIX_EPOCH};

/// One immutable operation instant, captured through a fallible injected clock.
/// Subsecond instants floor to Unix seconds, including before the epoch. Zone
/// observation/resolution is separate; no ambient zone is read by this value.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OperationTime {
    pub(super) seconds: i64,
}
impl OperationTime {
    pub fn capture(clock: impl FnOnce() -> Result<SystemTime, String>) -> Result<Self, String> {
        Ok(Self {
            seconds: unix_seconds(clock()?)?,
        })
    }
    pub fn unix_seconds(self) -> i64 {
        self.seconds
    }
    /// Apply a known offset, not a guessed zone rule. A zone resolver must first
    /// select the offset for this instant; transitions/folds are not inferred here.
    pub fn civil_at_offset(self, offset_seconds: i32) -> Result<CivilDateTime, String> {
        if !(-86400..=86400).contains(&offset_seconds) {
            return Err("Org UTC offset exceeds 24 hours".into());
        }
        let seconds = self
            .seconds
            .checked_add(i64::from(offset_seconds))
            .ok_or("Org operation time overflows")?;
        CivilDateTime::from_seconds(seconds)
    }
    /// Elapsed seconds between captured instants become work minutes. This does
    /// not subtract zone-less wall times or apply configured calendar units.
    pub fn elapsed_since(self, earlier: Self) -> Result<Duration, String> {
        let seconds = self
            .seconds
            .checked_sub(earlier.seconds)
            .ok_or("Org elapsed time overflows")?;
        Duration::from_minutes(seconds as f64 / 60.0)
    }
}
fn unix_seconds(time: SystemTime) -> Result<i64, String> {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_secs()).map_err(|_| "Org clock overflows".into()),
        Err(error) => {
            let duration = error.duration();
            let seconds = i64::try_from(duration.as_secs()).map_err(|_| "Org clock overflows")?;
            let fraction = i64::from(duration.subsec_nanos() != 0);
            seconds
                .checked_neg()
                .and_then(|s| s.checked_sub(fraction))
                .ok_or_else(|| "Org clock overflows".into())
        }
    }
}
