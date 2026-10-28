//! Bounded immutable calendar observations. Rows never carry edit authority.
mod observe;
mod project;
use super::{SourceIssue, SourceSet, SourceVersion};
use crate::{
    config::{AgendaSpan, Org},
    org::{
        Cancellation,
        time::{CivilTime, Date, OrgDay, ZonedOperation},
    },
};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug)]
pub struct CalendarQuery {
    pub operation: Arc<ZonedOperation>,
    pub today: Date,
    pub start: Date,
    pub span: AgendaSpan,
    pub week_start: usize,
    pub zone: String,
}
impl CalendarQuery {
    pub fn new(operation: ZonedOperation, policy: &Org) -> Result<Self, String> {
        let today = OrgDay::new(policy.time.extend_today_until as u8)?.date_at(operation.now())?;
        let value = Self {
            operation: Arc::new(operation),
            today,
            start: today,
            span: policy.agenda.span,
            week_start: policy.agenda.week_start,
            zone: policy.time.zone.clone(),
        };
        value.at(today, value.span)
    }
    pub fn at(&self, date: Date, span: AgendaSpan) -> Result<Self, String> {
        let mut value = self.clone();
        value.span = span;
        let offset = if span == AgendaSpan::Week {
            (i64::from(date.weekday()) - self.week_start as i64).rem_euclid(7)
        } else {
            0
        };
        value.start = date.add_days(-offset)?;
        value.end()?;
        Ok(value)
    }
    pub fn days(&self) -> usize {
        if self.span == AgendaSpan::Day { 1 } else { 7 }
    }
    pub fn end(&self) -> Result<Date, String> {
        self.start.add_days(self.days() as i64 - 1)
    }
    pub fn shifted(&self, direction: i64) -> Result<Self, String> {
        self.at(
            self.start.add_days(
                direction
                    .checked_mul(self.days() as i64)
                    .ok_or("Calendar span shift overflow")?,
            )?,
            self.span,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CalendarKind {
    OverdueScheduled,
    OverdueDeadline,
    Warning,
    Scheduled,
    Deadline,
    Appointment,
}
impl CalendarKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::OverdueScheduled => "Overdue scheduled",
            Self::OverdueDeadline => "Overdue deadline",
            Self::Warning => "Deadline warning",
            Self::Scheduled => "Scheduled",
            Self::Deadline => "Deadline",
            Self::Appointment => "Appointment",
        }
    }
}
#[derive(Clone, Debug)]
pub struct CalendarSource {
    pub path: Option<PathBuf>,
    pub version: SourceVersion,
}
#[derive(Clone, Debug)]
pub struct CalendarRow {
    pub date: Date,
    pub time: Option<CivilTime>,
    pub end_time: Option<CivilTime>,
    pub kind: CalendarKind,
    pub title: Arc<str>,
    pub source: usize,
    pub byte: usize,
    pub unresolved: bool,
    pub stale: bool,
}
#[derive(Clone, Debug)]
pub enum CalendarLine {
    Date(Date),
    Section(CalendarKind),
    Grid(CivilTime),
    Entry(usize),
    Issue(usize),
}
struct Event {
    source: usize,
    byte: usize,
    title: Arc<str>,
    value: crate::org::time::TimestampRange,
    kind: CalendarKind,
    done: bool,
}
struct Facts {
    events: Vec<Event>,
    sources: Vec<CalendarSource>,
    issues: Vec<SourceIssue>,
    truncated: bool,
    limit: usize,
    seconds: usize,
}
pub struct Calendar {
    pub query: CalendarQuery,
    pub rows: Vec<CalendarRow>,
    pub lines: Vec<CalendarLine>,
    pub sources: Vec<CalendarSource>,
    pub issues: Vec<SourceIssue>,
    pub truncated: bool,
    pub cancelled: bool,
    facts: Arc<Facts>,
}
impl Calendar {
    pub fn observe(
        sources: &SourceSet,
        query: CalendarQuery,
        policy: &Org,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        if !(1..=100000).contains(&policy.limits.agenda_rows)
            || !(1..=600).contains(&policy.limits.job_seconds)
        {
            return Err("Calendar requires bounded row/time policies".into());
        }
        let bounded = cancel.bounded(
            std::time::Instant::now()
                + std::time::Duration::from_secs(policy.limits.job_seconds as u64),
        );
        let facts = Arc::new(observe::sources(sources, policy, &bounded));
        Ok(project::calendar(facts, query, &bounded))
    }
    pub fn project(&self, query: CalendarQuery, cancel: &Cancellation) -> Self {
        project::calendar(self.facts.clone(), query, cancel)
    }
    pub fn complete(&self) -> bool {
        !self.truncated && !self.cancelled && self.issues.is_empty()
    }
    pub fn revalidate(&mut self, buffers: &crate::buffer::Buffers) {
        let stale: Vec<_> = self
            .sources
            .iter()
            .map(|s| match s.version {
                SourceVersion::Buffer(stamp) => {
                    !buffers.get(stamp.buffer).is_ok_and(|b| b.stamp() == stamp)
                }
                SourceVersion::Disk(_) => false,
            })
            .collect();
        for row in &mut self.rows {
            row.stale = stale[row.source];
        }
        for (i, _) in stale.iter().enumerate().filter(|(_, s)| **s) {
            self.issue(
                self.sources[i].path.clone(),
                "Source buffer changed; refresh required".into(),
            );
        }
        for index in self
            .lines
            .iter()
            .filter_map(|line| {
                if let CalendarLine::Issue(i) = line {
                    Some(*i)
                } else {
                    None
                }
            })
            .max()
            .map_or(0, |i| i + 1)..self.issues.len()
        {
            self.lines.push(CalendarLine::Issue(index));
        }
    }
    fn issue(&mut self, path: Option<PathBuf>, message: String) {
        if self.issues.len() < 4096 {
            self.issues.push(SourceIssue { path, message });
        } else {
            self.truncated = true;
        }
    }
}
