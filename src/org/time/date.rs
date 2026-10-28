//! Proleptic Gregorian dates in the source timestamp range 0001..9999.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Date {
    days: i32,
}
impl Date {
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, String> {
        if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
            return Err("Org date requires year 0001..9999 and month 01..12".into());
        }
        if day == 0 || day > month_days(year, month) {
            return Err("Invalid Gregorian day for Org date".into());
        }
        Ok(Self {
            days: ordinal(year, month, day),
        })
    }
    pub fn year(self) -> u16 {
        self.parts().0
    }
    pub fn month(self) -> u8 {
        self.parts().1
    }
    pub fn day(self) -> u8 {
        self.parts().2
    }
    /// Monday is 1, Sunday is 7, independently of locale and host time zone.
    pub fn weekday(self) -> u8 {
        (self.days + 3).rem_euclid(7) as u8 + 1
    }
    pub(super) fn from_days(days: i64) -> Result<Self, String> {
        if !(-719162..=2932896).contains(&days) {
            return Err("Org date is outside 0001..9999".into());
        }
        Ok(Self { days: days as i32 })
    }
    pub(super) fn days(self) -> i64 {
        i64::from(self.days)
    }
    fn parts(self) -> (u16, u8, u8) {
        let civil = i64::from(self.days) + 719468;
        let era = civil.div_euclid(146097);
        let day = civil - era * 146097;
        let year = (day - day / 1460 + day / 36524 - day / 146096) / 365;
        let day_year = day - (365 * year + year / 4 - year / 100);
        let month = (5 * day_year + 2) / 153;
        let d = day_year - (153 * month + 2) / 5 + 1;
        let m = month + if month < 10 { 3 } else { -9 };
        let y = year + era * 400 + i64::from(m <= 2);
        (y as u16, m as u8, d as u8)
    }
}
fn month_days(year: u16, month: u8) -> u8 {
    match month {
        2 => 28 + u8::from(leap_year(year)),
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
fn leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
fn ordinal(year: u16, month: u8, day: u8) -> i32 {
    let y = i32::from(year) - i32::from(month <= 2);
    let era = y.div_euclid(400);
    let yr = y - era * 400;
    let m = i32::from(month) + if month > 2 { -3 } else { 9 };
    let d = (153 * m + 2) / 5 + i32::from(day) - 1;
    era * 146097 + yr * 365 + yr / 4 - yr / 100 + d - 719468
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (y, m, d) = self.parts();
        write!(f, "{y:04}-{m:02}-{d:02}")
    }
}
