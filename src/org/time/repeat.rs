//! Checked source recurrence. No clock or zone is observed here; a completion
//! supplies its retained civil time and resolves the returned dates separately.
mod advance;
mod interval;
mod parse;

use super::{CivilDateTime, Timestamp};
pub use interval::{RepeatInterval, RepeatUnit};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepeatMode {
    Once,
    CatchUp,
    Restart,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Repeater {
    pub mode: RepeatMode,
    pub interval: RepeatInterval,
    pub maximum: Option<RepeatInterval>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WarningDelay {
    pub interval: RepeatInterval,
    pub first_only: bool,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Suffixes {
    pub repeater: Option<Repeater>,
    pub warning: Option<WarningDelay>,
}
impl Repeater {
    /// Minimum/maximum habit bounds retain calendar units. An inverted range
    /// is diagnosed at use; its original source remains round-trippable.
    pub fn habit_window(
        self,
        completed: CivilDateTime,
    ) -> Result<(CivilDateTime, CivilDateTime), String> {
        let first = self.interval.shift(completed, 1)?;
        let last = self.maximum.unwrap_or(self.interval).shift(completed, 1)?;
        if last < first {
            return Err("Org habit maximum interval precedes its minimum".into());
        }
        Ok((first, last))
    }
}
impl Timestamp {
    pub fn repeater(&self) -> Result<Option<Repeater>, String> {
        Ok(parse::suffixes(self.suffix())?.repeater)
    }
    pub fn warning_delay(&self) -> Result<Option<WarningDelay>, String> {
        Ok(parse::suffixes(self.suffix())?.warning)
    }
    /// Return a new active occurrence, or None when no active repeater applies.
    /// Inactive timestamps never advance. Rejection leaves the receiver exact.
    pub fn advance_repeat(&self, completed: CivilDateTime) -> Result<Option<Self>, String> {
        advance::timestamp(self, completed)
    }
    /// A deadline's explicit prewarning boundary, without agenda/source effects.
    pub fn deadline_warning(&self) -> Result<Option<CivilDateTime>, String> {
        self.warning_delay()?
            .map(|w| w.interval.shift(advance::civil(self)?, -1))
            .transpose()
    }
    /// The selected occurrence's delayed scheduled display boundary. A first-only
    /// delay disappears from source on a successful repeat, unlike ordinary '-'.
    pub fn scheduled_display(&self) -> Result<Option<CivilDateTime>, String> {
        self.warning_delay()?
            .map(|w| w.interval.shift(advance::civil(self)?, 1))
            .transpose()
    }
}
pub(super) fn check_suffix(text: &str) -> Result<(), String> {
    parse::suffixes(text).map(|_| ())
}

impl super::TimestampRange {
    /// Advance each repeating endpoint independently, preserving other source.
    /// An inverted candidate rejects the complete range before publication.
    pub fn advance_repeat(&self, completed: CivilDateTime) -> Result<Option<Self>, String> {
        advance::range(self, completed)
    }
}
