use super::{Date, date::month_days};

/// A fixed six-week projection. Empty cells never denote a neighboring source
/// date, and selecting a hidden/clipped cell cannot manufacture an edit target.
pub(crate) struct Month {
    first: Date,
    offset: u8,
    days: u8,
}
impl Month {
    pub(crate) fn new(selected: Date, shift: i64) -> Result<Self, String> {
        let first = Date::new(selected.year(), selected.month(), 1)?.add_months(shift)?;
        Ok(Self {
            first,
            offset: first.weekday() % 7,
            days: month_days(first.year(), first.month()),
        })
    }
    pub(crate) fn selected_day(&self, day: u8) -> Result<Date, String> {
        Date::new(self.first.year(), self.first.month(), day.min(self.days))
    }
    pub(crate) fn label(&self) -> String {
        let name = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ][usize::from(self.first.month() - 1)];
        format!("{name} {}", self.first.year())
    }
    pub(crate) fn cell(&self, row: u16, column: u16) -> Option<Date> {
        if row >= 6 || column >= 7 {
            return None;
        }
        let index = row * 7 + column;
        let day = index.checked_sub(u16::from(self.offset))? + 1;
        if day > u16::from(self.days) {
            return None;
        }
        Date::new(self.first.year(), self.first.month(), day as u8).ok()
    }
}
pub(crate) fn calendar_hit(selected: Date, cell: (u16, u16), width: u16) -> Option<Date> {
    let (column, row) = cell;
    if column >= width || (width >= 88 && column >= 88) {
        return None;
    }
    let row = row.checked_sub(2)?;
    let (month, column) = if width >= 88 {
        (i64::from(column / 30) - 1, column % 30)
    } else {
        (0, column)
    };
    if column >= 28 {
        return None;
    }
    Month::new(selected, month).ok()?.cell(row, column / 4)
}
