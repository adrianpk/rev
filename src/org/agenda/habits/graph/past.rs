use super::*;
pub(super) struct Past {
    scheduled: Date,
    slack: i64,
    dates: Vec<Date>,
    after: Vec<Date>,
}
impl Past {
    pub(super) fn new(record: &HabitRecord) -> Result<Self, String> {
        if record.completions.len() > 4096 {
            return Err("Habit graph exceeds 4096 source completions".into());
        }
        let repeat = record.schedule.repeat;
        let period = days(repeat.interval)?;
        let slack = days(repeat.maximum.unwrap_or(repeat.interval))? - period;
        if repeat.maximum.is_some() && slack <= 0 {
            return Err("Habit graph maximum must exceed its daily minimum".into());
        }
        let mut dates: Vec<_> = record.completions.iter().map(|v| v.date()).collect();
        dates.sort();
        let scheduled = record.schedule.scheduled.date();
        let after = history(&dates, scheduled, period, repeat.mode)?;
        Ok(Self {
            scheduled,
            slack,
            dates,
            after,
        })
    }
    pub(super) fn cell(&self, date: Date, today: Date, green: bool) -> Result<HabitCell, String> {
        let count = self.dates.partition_point(|d| *d < date);
        let completed = self.dates.get(count) == Some(&date);
        let scheduled = self.scheduled_on(date, today, count)?;
        let deadline = scheduled.add_days(self.slack)?;
        let state = if date < today && count == 0 && self.scheduled >= today {
            if completed {
                HabitState::Ready
            } else {
                HabitState::Clear
            }
        } else {
            state(date, scheduled, deadline, completed, green)
        };
        Ok(HabitCell {
            date,
            state,
            completed,
            today: date == today,
            future: date > today,
        })
    }
    fn scheduled_on(&self, date: Date, today: Date, count: usize) -> Result<Date, String> {
        if date >= today || count == 0 || count == self.dates.len() {
            return Ok(self.scheduled);
        }
        Ok(self.after[count - 1])
    }
}
fn state(date: Date, scheduled: Date, deadline: Date, done: bool, green: bool) -> HabitState {
    if date < scheduled {
        return HabitState::Clear;
    }
    if date < deadline || done && (date == deadline || green) {
        return HabitState::Ready;
    }
    if date == deadline {
        HabitState::Alert
    } else {
        HabitState::Overdue
    }
}
fn history(
    dates: &[Date],
    scheduled: Date,
    period: i64,
    mode: RepeatMode,
) -> Result<Vec<Date>, String> {
    let Some(first) = dates.first() else {
        return Ok(Vec::new());
    };
    let remainder = scheduled.days_since(*first).rem_euclid(period);
    let mut next = first.add_days(if remainder == 0 { period } else { remainder })?;
    let mut result = Vec::with_capacity(dates.len());
    for (i, done) in dates.iter().enumerate() {
        let value = match mode {
            RepeatMode::Once => scheduled.add_days(-((dates.len() - i - 1) as i64) * period)?,
            RepeatMode::Restart => done.add_days(period)?,
            RepeatMode::CatchUp => {
                if i > 0 {
                    next = next.add_days((1 + done.days_since(next).max(0) / period) * period)?;
                }
                next
            }
        };
        result.push(value);
    }
    Ok(result)
}
