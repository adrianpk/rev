use super::{CivilDateTime, Date};

/// Signed calendar shifts. Months/years normalize excess days as Org timestamp
/// adjustment does (January 31 + one month can reach March). No work-unit scale
/// participates, and a civil shift does not resolve a zone or measure elapsed time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarInterval {
    Minutes(i64),
    Hours(i64),
    Days(i64),
    Weeks(i64),
    Months(i64),
    Years(i64),
}
impl Date {
    pub fn add_days(self, count: i64) -> Result<Self, String> {
        let days = self
            .days()
            .checked_add(count)
            .ok_or("Org day shift overflows")?;
        Self::from_days(days)
    }
    pub fn add_months(self, count: i64) -> Result<Self, String> {
        let index = i64::from(self.year() - 1) * 12 + i64::from(self.month() - 1);
        let index = index
            .checked_add(count)
            .ok_or("Org month shift overflows")?;
        if !(0..119988).contains(&index) {
            return Err("Org month shift is outside 0001..9999".into());
        }
        let first = Self::new((index / 12 + 1) as u16, (index % 12 + 1) as u8, 1)?;
        first.add_days(i64::from(self.day() - 1))
    }
    pub fn add_years(self, count: i64) -> Result<Self, String> {
        self.add_months(count.checked_mul(12).ok_or("Org year shift overflows")?)
    }
}
impl CivilDateTime {
    pub fn shift(self, interval: CalendarInterval) -> Result<Self, String> {
        match interval {
            CalendarInterval::Months(n) => Ok(Self {
                date: self.date.add_months(n)?,
                ..self
            }),
            CalendarInterval::Years(n) => Ok(Self {
                date: self.date.add_years(n)?,
                ..self
            }),
            _ => self.shift_seconds(interval.seconds()?),
        }
    }
    fn shift_seconds(self, count: i64) -> Result<Self, String> {
        let seconds = self
            .seconds()
            .checked_add(count)
            .ok_or("Org civil shift overflows")?;
        Self::from_seconds(seconds)
    }
}
impl CalendarInterval {
    fn seconds(self) -> Result<i64, String> {
        let (n, scale) = match self {
            Self::Minutes(n) => (n, 60),
            Self::Hours(n) => (n, 3600),
            Self::Days(n) => (n, 86400),
            Self::Weeks(n) => (n, 604800),
            _ => return Err("Calendar months and years require date arithmetic".into()),
        };
        n.checked_mul(scale)
            .ok_or_else(|| "Org calendar interval overflows".into())
    }
}
