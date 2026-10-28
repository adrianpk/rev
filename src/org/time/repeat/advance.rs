use super::super::{CivilTime, Date};
use super::*;

pub(super) fn civil(stamp: &Timestamp) -> Result<CivilDateTime, String> {
    Ok(CivilDateTime {
        date: stamp.date(),
        time: stamp.time().unwrap_or(CivilTime::new(0, 0, 0)?),
    })
}
pub(super) fn timestamp(
    stamp: &Timestamp,
    completed: CivilDateTime,
) -> Result<Option<Timestamp>, String> {
    let Some(repeat) = active_repeater(stamp)? else {
        return Ok(None);
    };
    let original = civil(stamp)?;
    let next = occurrence(original, completed, repeat, stamp.time().is_some())?;
    shifted_timestamp(stamp, next).map(Some)
}
fn active_repeater(stamp: &Timestamp) -> Result<Option<Repeater>, String> {
    if !stamp.active() {
        return Ok(None);
    }
    let Some(repeat) = stamp.repeater()? else {
        return Ok(None);
    };
    if repeat.interval.unit() == RepeatUnit::Hour && stamp.time().is_none() {
        return Err("Hourly Org repeat requires a source clock".into());
    }
    if repeat.maximum.is_some() {
        repeat.habit_window(civil(stamp)?)?;
    }
    Ok(Some(repeat))
}
fn occurrence(
    original: CivilDateTime,
    completed: CivilDateTime,
    repeat: Repeater,
    timed: bool,
) -> Result<CivilDateTime, String> {
    match repeat.mode {
        RepeatMode::Once => repeat.interval.shift(original, 1),
        RepeatMode::CatchUp => catch_up(original, completed, repeat.interval, timed),
        RepeatMode::Restart => restart(original, completed, repeat.interval),
    }
}
fn shifted_timestamp(stamp: &Timestamp, next: CivilDateTime) -> Result<Timestamp, String> {
    let mut result = stamp.clone();
    result.set_date(next.date);
    if stamp.time().is_some() {
        result.set_time(Some(next.time))?;
        result.set_end_time(shifted_end(stamp, civil(stamp)?, next)?)?;
    }
    result.remove_first_delay();
    result.end_civil()?;
    Ok(result)
}
fn restart(
    original: CivilDateTime,
    completed: CivilDateTime,
    interval: RepeatInterval,
) -> Result<CivilDateTime, String> {
    let time = if interval.unit() == RepeatUnit::Hour {
        CivilTime::new(completed.time.hour(), completed.time.minute(), 0)?
    } else {
        original.time
    };
    interval.shift(
        CivilDateTime {
            date: completed.date,
            time,
        },
        1,
    )
}
fn shifted_end(
    stamp: &Timestamp,
    original: CivilDateTime,
    next: CivilDateTime,
) -> Result<Option<CivilTime>, String> {
    stamp
        .end_civil()?
        .map(|end| {
            CivilDateTime::from_seconds(next.seconds() + end.seconds() - original.seconds())
                .map(|v| v.time)
        })
        .transpose()
}
pub(in crate::org::time::repeat) fn catch_up(
    base: CivilDateTime,
    completed: CivilDateTime,
    interval: RepeatInterval,
    timed: bool,
) -> Result<CivilDateTime, String> {
    let cutoff = if timed {
        completed
    } else {
        CivilDateTime {
            date: completed.date,
            time: CivilTime::new(23, 59, 59)?,
        }
    };
    if let Some(seconds) = interval.seconds() {
        let count = ((cutoff.seconds() - base.seconds()).div_euclid(seconds) + 1).max(1);
        return interval.shift(base, count);
    }
    calendar_catch_up(base, cutoff, interval)
}
fn calendar_catch_up(
    mut base: CivilDateTime,
    cutoff: CivilDateTime,
    interval: RepeatInterval,
) -> Result<CivilDateTime, String> {
    // Fixed units jump directly. Calendar normalization can change a month-end
    // anchor, so normalize first and jump only when every visited month is safe.
    // 4800 covers a full Gregorian month cycle; source years are finite too.
    for _ in 0..4800 {
        if safe_months(base.date, interval.months()) {
            return jump_months(base, cutoff, interval);
        }
        base = interval.shift(base, 1)?;
        if base > cutoff {
            return Ok(base);
        }
    }
    Err("Org calendar recurrence exceeds its 4800-step normalization bound".into())
}
fn safe_months(date: Date, step: i64) -> bool {
    (1..=12).all(|n| {
        let month = ((i64::from(date.month()) - 1 + n * step).rem_euclid(12) + 1) as u8;
        date.day() <= super::super::date::month_days(2001, month)
    })
}
fn jump_months(
    base: CivilDateTime,
    cutoff: CivilDateTime,
    interval: RepeatInterval,
) -> Result<CivilDateTime, String> {
    let months = |date: Date| i64::from(date.year()) * 12 + i64::from(date.month());
    let count = ((months(cutoff.date) - months(base.date)).div_euclid(interval.months())).max(1);
    let candidate = interval.shift(base, count)?;
    if candidate > cutoff {
        Ok(candidate)
    } else {
        interval.shift(candidate, 1)
    }
}

pub(super) fn range(
    value: &super::super::TimestampRange,
    completed: CivilDateTime,
) -> Result<Option<super::super::TimestampRange>, String> {
    let start = value.start.advance_repeat(completed)?;
    let end = value
        .end
        .as_ref()
        .map(|end| end.advance_repeat(completed))
        .transpose()?
        .flatten();
    if start.is_none() && end.is_none() {
        return Ok(None);
    }
    let end = end.or_else(|| value.end.clone());
    let result = super::super::TimestampRange {
        start: start.unwrap_or_else(|| value.start.clone()),
        end,
    };
    result.check()?;
    Ok(Some(result))
}
