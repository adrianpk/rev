pub(super) mod columns;
use super::*;
use crate::buffer::Buffer;
use crate::coordinates::{Coordinates, Scalar};
use crate::org::agenda::{ClockQuery, ClockRow, ClockScan, ClockScope};

impl Outline {
    pub(crate) fn clock_scan(
        &self,
        buffer: &Buffer,
        point: Scalar,
        query: ClockQuery,
        policy: &crate::config::Org,
    ) -> Result<ClockScan, String> {
        self.report_current(buffer)?;
        let selected = self.report_headings(buffer, point, query.scope)?;
        let range = self.report_range(&selected, query.scope, buffer.len_bytes());
        if range.len() > 262144 {
            return Err("Clock report exceeds 262144 UTF-8 bytes".into());
        }
        let offset = range.start;
        let text = source(buffer, range.clone())?;
        let source = buffer.byte_span(range.clone()).map_err(|e| e.to_string())?;
        let literals = self
            .literals
            .iter()
            .filter(|r| r.start < range.end && r.end > range.start)
            .cloned()
            .collect();
        let mut rows = Vec::new();
        let base = selected.first().copied().unwrap_or(0);
        for index in selected {
            let mut row = self.clock_row(buffer, index, policy)?;
            row.parent = row.parent.and_then(|i| i.checked_sub(base));
            rows.push(row);
        }
        Ok(ClockScan {
            source,
            text,
            offset,
            literals,
            rows,
            window: query.window,
        })
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
            byte: h.start,
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
}

fn source(buffer: &Buffer, range: Range<usize>) -> Result<String, String> {
    let span = buffer.byte_span(range).map_err(|e| e.to_string())?;
    buffer
        .slice(&span)
        .map(|s| s.to_string())
        .map_err(|e| e.to_string())
}
mod dynamic;
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Dynamic {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) body: Range<usize>,
    pub(super) header: Range<usize>,
    pub(super) name: Range<usize>,
}
