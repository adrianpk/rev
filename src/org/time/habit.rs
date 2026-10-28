//! Civil habit schedules and Org's explicit day boundary. No ambient clock,
//! history inference or source mutation belongs to these values.
use super::{CalendarInterval, CivilDateTime, Date, RepeatUnit, Repeater, Timestamp};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OrgDay {
    hour: u8,
}
impl OrgDay {
    /// Midnight is the factory boundary; 0..23 selects an extended Org day.
    pub fn new(hour: u8) -> Result<Self, String> {
        if hour > 23 {
            return Err("Org day boundary requires hour 0..23".into());
        }
        Ok(Self { hour })
    }
    pub fn date_at(self, now: CivilDateTime) -> Result<Date, String> {
        now.shift(CalendarInterval::Hours(-i64::from(self.hour)))
            .map(|v| v.date)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HabitSchedule {
    pub scheduled: Timestamp,
    pub repeat: Repeater,
}
impl HabitSchedule {
    /// Only an active repeating SCHEDULED timestamp defines a habit. Hourly
    /// recurrence is not a daily habit; an explicit maximum must exceed minimum.
    pub fn new(scheduled: Timestamp) -> Result<Self, String> {
        let repeat = scheduled
            .repeater()?
            .filter(|_| scheduled.active())
            .ok_or("Org habit requires active repeating SCHEDULED")?;
        if repeat.interval.unit() == RepeatUnit::Hour
            || repeat.maximum.is_some_and(|v| v.unit() == RepeatUnit::Hour)
        {
            return Err("Org habit intervals require at least one calendar day".into());
        }
        let value = Self { scheduled, repeat };
        let anchor = CivilDateTime {
            date: value.scheduled.date(),
            time: super::CivilTime::new(0, 0, 0)?,
        };
        value.bounds_after(anchor)?;
        Ok(value)
    }
    /// Preserve the calendar units of both bounds. This is a due window, not
    /// an elapsed duration or evidence that a completion occurred.
    pub fn bounds_after(&self, completed: CivilDateTime) -> Result<(Date, Date), String> {
        let (first, last) = self.repeat.habit_window(completed)?;
        if self.repeat.maximum.is_some() && first >= last {
            return Err("Org habit maximum interval must exceed its minimum".into());
        }
        Ok((first.date, last.date))
    }
}

impl super::TimestampRange {
    pub(crate) fn advance_habit_repeat(
        &self,
        now: CivilDateTime,
        day: Option<OrgDay>,
    ) -> Result<Option<Self>, String> {
        let Some(day) = day else {
            return self.advance_repeat(now);
        };
        let next = |stamp: &Timestamp| {
            let date = if stamp.time().is_none() {
                day.date_at(now)?
            } else {
                now.date
            };
            stamp.advance_repeat(CivilDateTime {
                date,
                time: now.time,
            })
        };
        let start = next(&self.start)?;
        let end = self.end.as_ref().map(next).transpose()?.flatten();
        if start.is_none() && end.is_none() {
            return Ok(None);
        }
        let result = Self {
            start: start.unwrap_or_else(|| self.start.clone()),
            end: end.or_else(|| self.end.clone()),
        };
        result.check()?;
        Ok(Some(result))
    }
}
