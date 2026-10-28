use super::{ClockQuery, ClockRow, ClockSummary};
use crate::{
    buffer::{Buffer, Span},
    coordinates::Scalar,
    org::{
        Cancellation, Structure,
        time::{CivilDateTime, CivilInterval, RecordedClock, Resolution, Zone},
    },
};
use std::{
    ops::Range,
    time::{Duration, Instant},
};

/// Explicit native report request. It carries no file access or edit authority.
pub struct ClockRequest<'a> {
    pub point: Scalar,
    pub query: ClockQuery,
    pub policy: &'a crate::config::Org,
}
/// An immutable bounded snapshot, suitable for the shared native Job owner.
/// Completion still needs the caller's source/view/read-only publication checks.
pub struct ClockScan {
    pub(crate) source: Span,
    pub(crate) text: String,
    pub(crate) offset: usize,
    pub(crate) literals: Vec<Range<usize>>,
    pub(crate) rows: Vec<ClockRow>,
    pub(crate) window: Option<CivilInterval>,
}
impl ClockScan {
    pub fn new(
        buffer: &Buffer,
        structure: &Structure,
        request: ClockRequest<'_>,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        crate::org::source::current(buffer, structure.stamp()).map_err(|e| e.to_string())?;
        let text = crate::org::Source::new(buffer)
            .snapshot()
            .map_err(|e| e.to_string())?;
        let model = crate::org::Outline::observe(structure, &text, cancel);
        model.clock_scan(buffer, request.point, request.query, request.policy)
    }
    pub fn run(self, zone: &Zone, cancel: &Cancellation) -> Result<ClockSummary, String> {
        let window = date_window(self.window, zone)?;
        let mut summary = ClockSummary {
            source: self.source,
            rows: self.rows,
            minutes: 0,
            effort: None,
            open: 0,
            unresolved: 0,
            overlaps: 0,
        };
        let mut records = Vec::new();
        let mut count = 0;
        let mut byte = self.offset;
        let mut work = Progress {
            cancel,
            yielded: Instant::now(),
        };
        for line in self.text.split_inclusive('\n') {
            work.tick()?;
            if line.trim_start().starts_with("CLOCK:") {
                count += 1;
                if count > 4096 {
                    return Err("Clock report exceeds 4096 records".into());
                }
                observe(
                    &mut summary,
                    &mut records,
                    (byte, line),
                    (window, zone, &self.literals),
                )?;
            }
            byte += line.len();
        }
        summary.finish(&mut records)?;
        work.tick()?;
        Ok(summary)
    }
}
pub(crate) struct Progress<'a> {
    pub cancel: &'a Cancellation,
    pub yielded: Instant,
}
impl Progress<'_> {
    pub(crate) fn tick(&mut self) -> Result<(), String> {
        if self.cancel.is_cancelled() {
            return Err("Org clock report cancelled".into());
        }
        if self.yielded.elapsed() >= Duration::from_millis(10) {
            std::thread::yield_now();
            self.yielded = Instant::now();
        }
        Ok(())
    }
}
fn observe(
    summary: &mut ClockSummary,
    records: &mut Vec<RecordedClock>,
    line: (usize, &str),
    data: (Option<RecordedClock>, &Zone, &[Range<usize>]),
) -> Result<(), String> {
    let (window, zone, literals) = data;
    let index = literals.partition_point(|r| r.end <= line.0);
    if literals.get(index).is_some_and(|r| r.contains(&line.0)) {
        return Ok(());
    }
    let row = summary
        .rows
        .partition_point(|r| r.byte <= line.0)
        .checked_sub(1);
    let Some(row) = row else { return Ok(()) };
    let row = &mut summary.rows[row];
    match RecordedClock::observe(line.1, zone) {
        Ok(Some(record)) => add_record(row, records, record.clipped(window))?,
        Ok(None) => row.open += 1,
        Err(_) => row.unresolved += 1,
    }
    Ok(())
}
fn add_record(
    row: &mut ClockRow,
    records: &mut Vec<RecordedClock>,
    record: Option<RecordedClock>,
) -> Result<(), String> {
    if let Some(record) = record {
        row.minutes = row
            .minutes
            .checked_add(record.minutes())
            .ok_or("Clock total overflow")?;
        records.push(record);
    }
    Ok(())
}
fn date_window(
    window: Option<CivilInterval>,
    zone: &Zone,
) -> Result<Option<RecordedClock>, String> {
    let Some(window) = window else {
        return Ok(None);
    };
    Ok(Some(RecordedClock {
        start: boundary(window.start(), zone)?,
        end: boundary(window.end(), zone)?,
    }))
}
fn boundary(value: CivilDateTime, zone: &Zone) -> Result<i64, String> {
    match zone.resolve(value)? {
        Resolution::Unique(value) => Ok(value.unix_seconds.div_euclid(60)),
        _ => Err("Clock date boundary is ambiguous or nonexistent in the selected zone".into()),
    }
}
