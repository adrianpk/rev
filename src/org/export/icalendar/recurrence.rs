use super::*;
use crate::org::time::{RepeatMode, RepeatUnit, Repeater};
impl Calendar<'_> {
    pub(super) fn recurrence(
        &mut self,
        value: &TimestampRange,
        limitation: Option<&str>,
    ) -> Result<(), String> {
        if value.start.warning_delay()?.is_some() {
            self.diagnostic("Calendar warning delay retained in source; no alarm generated")?;
        }
        let Some(repeat) = value.start.repeater()? else {
            return Ok(());
        };
        let rule = limitation.map_or_else(|| self.rule(value, repeat), Err);
        match rule {
            Ok(rule) => self.raw("RRULE", &rule),
            Err(reason) => self.limit(reason),
        }
    }
    fn limit(&mut self, reason: &str) -> Result<(), String> {
        self.property("X-ORG-RECURRENCE-LIMIT", reason)?;
        self.diagnostic(&format!(
            "Calendar recurrence retained as source without RRULE: {reason}"
        ))
    }
    pub(super) fn todo_rule(&mut self, entry: &Entry) -> Result<(), String> {
        let limitation = if entry.scheduled.is_some() && entry.deadline.is_some() {
            Some("multiple task planning anchors")
        } else if entry.scheduled.is_none() {
            Some("task recurrence has no DTSTART")
        } else {
            None
        };
        for value in [&entry.scheduled, &entry.deadline].into_iter().flatten() {
            self.recurrence(value, limitation)?;
        }
        Ok(())
    }
    fn rule(&self, value: &TimestampRange, repeat: Repeater) -> Result<String, &'static str> {
        let frequency = admission(value, repeat)?;
        let stamp = &value.start;
        if stamp.time().is_some() && self.zone.constant_offset().is_none() {
            return Err("timed recurrence needs variable-zone rules, not a UTC interval");
        }
        self.calendar_fields(value, repeat.interval.unit())?;
        Ok(format!(
            "FREQ={frequency};INTERVAL={}",
            repeat.interval.count()
        ))
    }
    fn calendar_fields(
        &self,
        value: &TimestampRange,
        unit: RepeatUnit,
    ) -> Result<(), &'static str> {
        if !matches!(unit, RepeatUnit::Month | RepeatUnit::Year) {
            return Ok(());
        }
        let stamp = &value.start;
        if let Some(civil) = stamp.civil() {
            let instant = match self.zone.resolve(civil) {
                Ok(Resolution::Unique(v)) => v.unix_seconds,
                _ => return Err("unresolved recurrence clock"),
            };
            let utc = crate::org::time::OperationTime::from_unix_seconds(instant)
                .civil_at_offset(0)
                .map_err(|_| "recurrence UTC date outside native range")?;
            if utc.date != stamp.date() {
                return Err("UTC conversion changes monthly/yearly calendar fields");
            }
        }
        Ok(())
    }
}
fn admission(value: &TimestampRange, repeat: Repeater) -> Result<&'static str, &'static str> {
    if repeat.mode != RepeatMode::Once {
        return Err("completion/catch-up repeat mode");
    }
    if repeat.maximum.is_some() {
        return Err("habit minimum/maximum interval");
    }
    if value.end.is_some() {
        return Err("independently repeating date-range endpoints");
    }
    if repeat.interval.count() > i32::MAX as u32 {
        return Err("INTERVAL exceeds iCalendar integer range");
    }
    frequency(value, repeat.interval.unit())
}
fn frequency(value: &TimestampRange, unit: RepeatUnit) -> Result<&'static str, &'static str> {
    safe_date(value, unit)?;
    Ok(match unit {
        RepeatUnit::Hour => "HOURLY",
        RepeatUnit::Day => "DAILY",
        RepeatUnit::Week => "WEEKLY",
        RepeatUnit::Month => "MONTHLY",
        RepeatUnit::Year => "YEARLY",
    })
}
fn safe_date(value: &TimestampRange, unit: RepeatUnit) -> Result<(), &'static str> {
    let date = value.start.date();
    match unit {
        RepeatUnit::Hour if value.start.time().is_none() => {
            Err("hourly recurrence without a clock")
        }
        RepeatUnit::Month if date.day() > 28 => {
            Err("Org month normalization differs from skipped iCalendar dates")
        }
        RepeatUnit::Year if date.month() == 2 && date.day() == 29 => {
            Err("Org leap-day normalization differs from skipped iCalendar dates")
        }
        _ => Ok(()),
    }
}
