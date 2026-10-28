use super::super::{CalendarInterval, CivilDateTime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepeatUnit {
    Hour,
    Day,
    Week,
    Month,
    Year,
}
/// Positive source count. Constructors prevent zero intervals and all shift
/// multiplication is checked before calendar arithmetic or source publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepeatInterval {
    count: u32,
    unit: RepeatUnit,
}
impl RepeatInterval {
    pub fn new(count: u32, unit: RepeatUnit) -> Result<Self, String> {
        if count == 0 {
            return Err("Org interval count must be positive".into());
        }
        Ok(Self { count, unit })
    }
    pub fn count(self) -> u32 {
        self.count
    }
    pub fn unit(self) -> RepeatUnit {
        self.unit
    }
    pub fn shift(self, value: CivilDateTime, multiples: i64) -> Result<CivilDateTime, String> {
        let amount = i64::from(self.count)
            .checked_mul(multiples)
            .ok_or("Org repeat interval multiplication overflows")?;
        let shift = match self.unit {
            RepeatUnit::Hour => CalendarInterval::Hours(amount),
            RepeatUnit::Day => CalendarInterval::Days(amount),
            RepeatUnit::Week => CalendarInterval::Weeks(amount),
            RepeatUnit::Month => CalendarInterval::Months(amount),
            RepeatUnit::Year => CalendarInterval::Years(amount),
        };
        value.shift(shift)
    }
    pub(super) fn seconds(self) -> Option<i64> {
        let unit = match self.unit {
            RepeatUnit::Hour => 3600,
            RepeatUnit::Day => 86400,
            RepeatUnit::Week => 604800,
            _ => return None,
        };
        Some(i64::from(self.count) * unit)
    }
    pub(super) fn months(self) -> i64 {
        i64::from(self.count) * if self.unit == RepeatUnit::Year { 12 } else { 1 }
    }
}
