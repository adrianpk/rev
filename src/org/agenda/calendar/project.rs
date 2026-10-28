use super::*;
use crate::org::time::{CivilDateTime, CivilTime, Resolution, Timestamp, TimestampRange};
use std::time::{Duration, Instant};
pub(super) fn calendar(facts: Arc<Facts>, query: CalendarQuery, cancel: &Cancellation) -> Calendar {
    let bounded = cancel.bounded(Instant::now() + Duration::from_secs(facts.seconds as u64));
    let mut view = Calendar {
        query,
        rows: Vec::new(),
        lines: Vec::new(),
        sources: facts.sources.clone(),
        issues: facts.issues.clone(),
        truncated: facts.truncated,
        cancelled: false,
        facts: facts.clone(),
    };
    for event in &facts.events {
        if bounded.is_cancelled() {
            break;
        }
        if let Err(error) = event_rows(&mut view, event, &bounded) {
            view.issue(view.sources[event.source].path.clone(), error);
        }
        if view.rows.len() == facts.limit && view.truncated {
            break;
        }
        std::thread::yield_now();
    }
    view.cancelled = bounded.is_cancelled() && !bounded.expired();
    if bounded.expired() {
        view.truncated = true;
        view.issue(None, "Calendar job time limit reached".into());
    }
    view.rows
        .sort_by_key(|r| (r.date, r.time, r.kind, r.source, r.byte));
    view.lines = lines(&view);
    view
}
fn event_rows(view: &mut Calendar, event: &Event, cancel: &Cancellation) -> Result<(), String> {
    if event.kind != CalendarKind::Appointment {
        return planning(view, event);
    }
    let mut value = event.value.clone();
    if value.end.is_none() {
        value.start = value.start.occurrence_from(view.query.start)?;
    }
    for _ in 0..4096 {
        if cancel.is_cancelled() {
            return Ok(());
        }
        range_rows(view, event, &value)?;
        if !next_occurrence(&mut value, view.query.end()?)? {
            return Ok(());
        }
    }
    Err("Calendar recurrence exceeds 4096 projected occurrences".into())
}
fn next_occurrence(value: &mut TimestampRange, end: Date) -> Result<bool, String> {
    let Some(repeat) = value.start.repeater()? else {
        return Ok(false);
    };
    if value.end.is_some() {
        return Ok(false);
    }
    let base = CivilDateTime {
        date: value.start.date(),
        time: value.start.time().unwrap_or(CivilTime::new(0, 0, 0)?),
    };
    let next = repeat.interval.shift(base, 1)?;
    if next.date > end {
        return Ok(false);
    }
    value.start.set_date(next.date);
    if value.start.time().is_some() {
        value.start.set_time(Some(next.time))?;
    }
    Ok(true)
}
fn range_rows(view: &mut Calendar, event: &Event, value: &TimestampRange) -> Result<(), String> {
    let first = value.start.date();
    let last = value
        .end
        .as_ref()
        .map(|s| s.date())
        .or(value.start.end_civil()?.map(|v| v.date))
        .unwrap_or(first);
    let mut date = first.max(view.query.start);
    let end = last.min(view.query.end()?);
    while date <= end {
        push(view, event, date, event.kind, &value.start)?;
        if date == end {
            break;
        }
        date = date.add_days(1)?;
    }
    Ok(())
}
fn planning(view: &mut Calendar, event: &Event) -> Result<(), String> {
    let stamp = &event.value.start;
    if original_date(&view.query, event)? {
        push(view, event, stamp.date(), event.kind, stamp)?;
    }
    if let Some(kind) = current_reminder(&view.query, event)? {
        push(view, event, view.query.today, kind, stamp)?;
    }

    Ok(())
}
fn current_reminder(query: &CalendarQuery, event: &Event) -> Result<Option<CalendarKind>, String> {
    if event.done || query.today < query.start || query.today > query.end()? {
        return Ok(None);
    }
    reminder(event, query.today)
}
fn original_date(query: &CalendarQuery, event: &Event) -> Result<bool, String> {
    let stamp = &event.value.start;
    let date = stamp.date();
    if date < query.start || date > query.end()? {
        return Ok(false);
    }
    if event.kind == CalendarKind::Scheduled {
        return Ok(stamp.scheduled_display()?.is_none_or(|v| v.date <= date));
    }
    Ok(true)
}
fn reminder(event: &Event, today: Date) -> Result<Option<CalendarKind>, String> {
    let stamp = &event.value.start;
    let date = stamp.date();
    if event.kind == CalendarKind::Scheduled {
        let start = stamp.scheduled_display()?.map_or(date, |v| v.date);
        return Ok((date < today && start <= today).then_some(CalendarKind::OverdueScheduled));
    }
    if date < today {
        return Ok(Some(CalendarKind::OverdueDeadline));
    }
    if date == today {
        return Ok(None);
    }
    let warning = stamp.deadline_warning()?.map_or_else(
        || {
            date.add_days(-14)
                .unwrap_or(Date::new(1, 1, 1).expect("minimum date"))
        },
        |v| v.date,
    );
    Ok((warning <= today).then_some(CalendarKind::Warning))
}

fn push(
    view: &mut Calendar,
    event: &Event,
    date: Date,
    kind: CalendarKind,
    stamp: &Timestamp,
) -> Result<(), String> {
    if view.rows.len() >= view.facts.limit {
        view.truncated = true;
        return Err("Calendar row limit reached".into());
    }
    let time = if kind == CalendarKind::Appointment || date == stamp.date() {
        stamp.time()
    } else {
        None
    };
    let unresolved = if let Some(time) = time {
        !matches!(
            view.query
                .operation
                .zone()
                .resolve(CivilDateTime { date, time })?,
            Resolution::Unique(_)
        )
    } else {
        false
    };
    view.rows.push(CalendarRow {
        date,
        time,
        end_time: stamp.end_time(),
        kind,
        title: event.title.clone(),
        source: event.source,
        byte: event.byte,
        unresolved,
        stale: false,
    });
    Ok(())
}
pub(super) fn lines(view: &Calendar) -> Vec<CalendarLine> {
    let mut lines = Vec::new();
    for offset in 0..view.query.days() {
        let Ok(date) = view.query.start.add_days(offset as i64) else {
            break;
        };
        lines.push(CalendarLine::Date(date));
        for kind in [
            CalendarKind::OverdueScheduled,
            CalendarKind::OverdueDeadline,
            CalendarKind::Warning,
            CalendarKind::Scheduled,
            CalendarKind::Deadline,
        ] {
            let entries: Vec<_> = view
                .rows
                .iter()
                .enumerate()
                .filter(|(_, r)| r.date == date && r.kind == kind)
                .map(|(i, _)| i)
                .collect();
            if !entries.is_empty() {
                lines.push(CalendarLine::Section(kind));
                lines.extend(entries.into_iter().map(CalendarLine::Entry));
            }
        }
        let mut timed: Vec<_> = view
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.date == date && r.kind == CalendarKind::Appointment)
            .map(|(i, r)| (r.time, CalendarLine::Entry(i)))
            .collect();
        if view.rows.iter().any(|r| r.date == date && r.time.is_some()) {
            for hour in (8..=20).step_by(2) {
                let t = CivilTime::new(hour, 0, 0).expect("grid hours");
                timed.push((Some(t), CalendarLine::Grid(t)));
            }
        }
        timed.sort_by_key(|(time, _)| *time);
        lines.extend(timed.into_iter().map(|(_, line)| line));
    }
    lines.extend((0..view.issues.len()).map(CalendarLine::Issue));
    lines
}
