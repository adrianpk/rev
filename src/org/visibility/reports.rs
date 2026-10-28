mod collect;
use super::*;
use crate::buffer::Buffer;
use crate::coordinates::{Coordinates, Scalar};
use crate::org::agenda::{ClockQuery, ClockRow, ClockScope, ClockSummary};
use crate::org::time::{CivilDateTime, CivilTime, RecordedClock, Resolution, Zone};

impl Outline {
    pub(crate) fn clock_summary(
        &self,
        buffer: &Buffer,
        point: Scalar,
        query: ClockQuery,
        data: (&crate::config::Org, &Zone),
    ) -> Result<ClockSummary, String> {
        self.report_current(buffer)?;
        let selected = self.report_headings(buffer, point, query.scope)?;
        let range = self.report_range(&selected, query.scope, buffer.len_bytes());
        if range.len() > 262144 {
            return Err("Clock report exceeds 262144 UTF-8 bytes".into());
        }
        let source = buffer.byte_span(range).map_err(|e| e.to_string())?;
        let window = date_window(query, data.1)?;
        let mut summary = ClockSummary {
            source,
            rows: Vec::new(),
            minutes: 0,
            effort: None,
            open: 0,
            unresolved: 0,
            overlaps: 0,
        };
        let mut records = Vec::new();
        for index in selected {
            let row = self.clock_row(buffer, index, data.0)?;
            summary.rows.push(row);
        }
        self.report_records(buffer, window, data.1, (&mut summary, &mut records))?;
        summary.overlaps = crate::org::agenda::clocks::overlaps(&mut records);
        self.sum_rows(&mut summary)?;
        Ok(summary)
    }
    fn report_current(&self, buffer: &Buffer) -> Result<(), String> {
        self.complete().map_err(str::to_owned)?;
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())
    }
    fn report_headings(
        &self,
        buffer: &Buffer,
        point: Scalar,
        scope: ClockScope,
    ) -> Result<Vec<usize>, String> {
        if scope == ClockScope::File {
            return Ok((0..self.headings.len()).collect());
        }
        let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let byte = c.to_byte(point).map_err(|e| e.to_string())?.0;
        let index = self
            .heading(byte)
            .ok_or("No Org heading for clock report")?;
        let end = if scope == ClockScope::Task {
            index + 1
        } else {
            self.headings
                .partition_point(|h| h.start < self.headings[index].end)
        };
        Ok((index..end).collect())
    }
    fn report_range(&self, selected: &[usize], scope: ClockScope, bytes: usize) -> Range<usize> {
        let Some(index) = selected.first().copied() else {
            return 0..bytes;
        };
        match scope {
            ClockScope::File => 0..bytes,
            ClockScope::Task => self.headings[index].start..self.headings[index].section,
            ClockScope::Subtree => self.headings[index].start..self.headings[index].end,
        }
    }
    fn clock_row(
        &self,
        buffer: &Buffer,
        index: usize,
        policy: &crate::config::Org,
    ) -> Result<ClockRow, String> {
        let h = &self.headings[index];
        let entry = self.heading_property_entry(buffer, index)?;
        let effort = self.property_value(buffer, entry, "EFFORT", &policy.properties.inherit)?;
        let effort = if effort.text.is_empty() {
            None
        } else {
            Some(crate::org::Duration::parse(
                &effort.text,
                &policy.duration.units,
            )?)
        };
        Ok(ClockRow {
            source: buffer
                .byte_span(h.start..h.section)
                .map_err(|e| e.to_string())?,
            title: source(buffer, h.title.clone())?,
            level: h.level,
            parent: h.parent,
            minutes: 0,
            effort,
            open: 0,
            unresolved: 0,
        })
    }
    fn sum_rows(&self, summary: &mut ClockSummary) -> Result<(), String> {
        for row in &summary.rows {
            summary.minutes = summary
                .minutes
                .checked_add(row.minutes)
                .ok_or("Clock total overflow")?;
            summary.open += row.open;
            summary.unresolved += row.unresolved;
        }
        // Compare a selected task/subtree with its own estimate; a file compares
        // top-level estimates, avoiding duplicate inherited parent estimates.
        for row in &summary.rows {
            if (row.parent.is_none() || row.source.range.start == summary.source.range.start)
                && let Some(effort) = row.effort
            {
                summary.effort = Some(
                    summary
                        .effort
                        .unwrap_or(crate::org::Duration::from_minutes(0.0)?)
                        .checked_add(effort)?,
                );
            }
        }
        Ok(())
    }
}
fn source(buffer: &Buffer, range: Range<usize>) -> Result<String, String> {
    let span = buffer.byte_span(range).map_err(|e| e.to_string())?;
    buffer
        .slice(&span)
        .map(|s| s.to_string())
        .map_err(|e| e.to_string())
}
fn date_window(query: ClockQuery, zone: &Zone) -> Result<Option<RecordedClock>, String> {
    let Some((start, end)) = query.dates else {
        return Ok(None);
    };
    let time = CivilTime::new(0, 0, 0)?;
    Ok(Some(RecordedClock {
        start: midnight(CivilDateTime { date: start, time }, zone)?,
        end: midnight(CivilDateTime { date: end, time }, zone)?,
    }))
}
fn midnight(value: CivilDateTime, zone: &Zone) -> Result<i64, String> {
    match zone.resolve(value)? {
        Resolution::Unique(value) => Ok(value.unix_seconds.div_euclid(60)),
        _ => Err("Clock date boundary is ambiguous or nonexistent in the selected zone".into()),
    }
}
