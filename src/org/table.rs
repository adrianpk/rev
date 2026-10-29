//! Native table plans retain original cell source. Only the canonical buffer
//! transaction can publish edits or grant history and view authority.
mod action;
mod calculate;
mod edit;
pub(crate) mod interchange;
mod layout;
mod observe;
mod references;

use super::{Cancellation, Coverage, Limits, Source, StructureKind as K, structure};
use crate::{
    buffer::{
        Buffer, Span, Stamp,
        edit::{Point, Replacement, Retained},
    },
    command::OrgCommand as C,
    coordinates::{Glyphs, Scalar},
};
use ropey::Rope;
use std::ops::Range;
const ROWS: usize = 10000;
const COLUMNS: usize = 256;
const CELLS: usize = 100000;
const WIDTH: usize = 4096;

pub(crate) struct Snapshot {
    pub stamp: Stamp,
    text: Rope,
    point: usize,
    tab_width: usize,
    newline: &'static str,
    region: Option<Range<usize>>,
    mark: Option<usize>,
    numeric: super::formula::eval::Policy,
    syntax: super::formula::Limits,
}
pub(crate) enum Output {
    Prompt,
    Plan(Plan),
    Fallback,
    Imported(Plan, Box<super::io::DestinationSource>),
    File(String, crate::file::Location),
    Exported(crate::file::Outcome),
}
pub(crate) struct Plan {
    pub edits: Vec<Replacement>,
    pub cursor: Option<Scalar>,
    pub retained: Vec<Retained>,
}
struct Table {
    source: Range<usize>,
    rows: Vec<Row>,
    columns: usize,
    formulas: Vec<Range<usize>>,
    column_order: Vec<Option<usize>>,
}
struct Row {
    old_row: Option<usize>,
    source: Range<usize>,
    body: Range<usize>,
    indent: String,
    ending: String,
    cells: Vec<Cell>,
    separator: bool,
    rule_columns: usize,
}
struct Cell {
    source: Range<usize>,
    value: String,
}
#[derive(Clone, Copy)]
struct Position {
    row: usize,
    column: usize,
    offset: usize,
    prefix: bool,
}
#[derive(Clone, Copy, PartialEq)]
enum Alignment {
    Left,
    Center,
    Right,
}
struct Anchor {
    source: Range<usize>,
    output: Range<usize>,
}
struct Layout {
    text: String,
    anchors: Vec<Anchor>,
    positions: Vec<Vec<Range<usize>>>,
    starts: Vec<usize>,
}

impl Snapshot {
    pub fn new(buffer: &Buffer, point: Point, org: &crate::config::Org) -> Result<Self, String> {
        let region = buffer.region(point).map_err(|e| e.to_string())?;
        let text = Source::new(buffer).snapshot().map_err(|e| e.to_string())?;
        if text.len_bytes() > Limits::default().bytes {
            return Err("Table source exceeds native analysis byte limit".into());
        }
        Ok(Self {
            stamp: buffer.stamp(),
            point: text.char_to_byte(point.position.0),
            region: region
                .as_ref()
                .map(|r| text.char_to_byte(r.range.start)..text.char_to_byte(r.range.end)),
            mark: region.and(buffer.mark()).map(|p| text.char_to_byte(p.0)),
            text,
            tab_width: buffer.policy().tab_width,
            newline: buffer.newline(),
            numeric: super::formula::eval::Policy {
                cells: org.limits.formula_cells,
                steps: org.limits.formula_steps,
                significant_digits: org.tables.significant_digits,
                decimal_places: org.limits.formula_decimal_places,
            },
            syntax: super::formula::Limits {
                steps: org.limits.formula_steps,
                depth: org.limits.formula_depth,
                decimal_places: org.limits.formula_decimal_places,
                ..super::formula::Limits::default()
            },
        })
    }
    pub fn run(
        self,
        command: C,
        argument: Option<String>,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        let raw = self.nodes(cancel)?;
        if command == C::ListToTable {
            return self.list_table(&raw, cancel).map(Output::Plan);
        }
        if command == C::TableCreate {
            return self.create(&raw, argument, cancel);
        }
        let at = self.calculation_context(&raw, command);
        let Some(table) = Table::observe(&self.text, &raw, at, cancel)? else {
            return if command.table() && command != C::Return {
                Err("No native Org table at point".into())
            } else {
                Ok(Output::Fallback)
            };
        };
        if table.split_line(command, self.point) {
            return Ok(Output::Fallback);
        }
        if self.calculation_requested(&table, command) {
            return self.recalculate(&table, command, cancel).map(Output::Plan);
        }
        let command = edit::command(command);
        if edit::structural(command) {
            return self.edit_table(table, command, argument, cancel);
        }
        self.operate(table, command, cancel).map(Output::Plan)
    }
    fn nodes(&self, cancel: &Cancellation) -> Result<Vec<structure::Raw>, String> {
        let raw = structure::observe(self.text.clone(), Limits::default(), cancel);
        if raw.coverage != Coverage::Complete || raw.nodes.iter().any(|n| n.kind == K::Malformed) {
            return Err("Incomplete or malformed table source; nothing changed".into());
        }
        Ok(raw.nodes)
    }
    fn plan(&self, table: &Table, layout: Layout, cursor: Position) -> Result<Plan, String> {
        let cell = layout
            .positions
            .get(cursor.row)
            .and_then(|r| r.get(cursor.column))
            .ok_or("Table destination has no cell")?;
        let target = if cursor.prefix {
            layout.starts[cursor.row]..cell.start
        } else {
            cell.clone()
        };
        let at = target.start
            + layout.text[target.clone()]
                .char_indices()
                .nth(cursor.offset)
                .map_or(target.len(), |(b, _)| b);
        let cursor =
            Scalar(self.text.byte_to_char(table.source.start) + layout.cursor(cursor.row, at)?);
        let mut edits = Vec::new();
        let mut source = table.source.start;
        let mut output = 0;
        for anchor in &layout.anchors {
            self.gap(
                &mut edits,
                source..anchor.source.start,
                &layout.text[output..anchor.output.start],
            )?;
            source = anchor.source.end;
            output = anchor.output.end;
        }
        self.gap(&mut edits, source..table.source.end, &layout.text[output..])?;
        Ok(Plan {
            edits,
            cursor: Some(cursor),
            retained: Vec::new(),
        })
    }
    fn gap(
        &self,
        edits: &mut Vec<Replacement>,
        bytes: Range<usize>,
        value: &str,
    ) -> Result<(), String> {
        if self.text.byte_slice(bytes.clone()) == value {
            return Ok(());
        }
        let range = self.text.byte_to_char(bytes.start)..self.text.byte_to_char(bytes.end);
        edits.push(Replacement {
            source: Span {
                stamp: self.stamp,
                range,
            },
            text: value.into(),
        });
        Ok(())
    }
}
impl Table {
    fn split_line(&self, command: C, point: usize) -> bool {
        matches!(command, C::Return | C::TableNextRow)
            && self
                .rows
                .iter()
                .any(|r| point == r.body.start || point == r.body.end)
    }
}
fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Org table action cancelled or timed out".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffers;
    use std::time::Instant;

    // Cancelled/expired analysis never yields a publication plan, and oversized
    // source is rejected while snapshotting the real canonical buffer identity.
    #[test]
    fn work_limits() {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create("|A|B|\n").unwrap();
        let buffer = buffers.get(id).unwrap();
        let point = Point {
            stamp: buffer.stamp(),
            position: Scalar(0),
        };
        let cancelled = Cancellation::default();
        cancelled.cancel();
        let expired = Cancellation::default().bounded(Instant::now());
        for (name, token) in [("cancelled", cancelled), ("expired", expired)] {
            let snapshot = Snapshot::new(buffer, point, &crate::config::Org::default()).unwrap();
            assert!(snapshot.run(C::TableAlign, None, &token).is_err(), "{name}");
            assert!(buffer.history().is_empty());
        }
        let id = buffers
            .create(&"x".repeat(Limits::default().bytes + 1))
            .unwrap();
        let buffer = buffers.get(id).unwrap();
        let point = Point {
            stamp: buffer.stamp(),
            position: Scalar(0),
        };
        assert!(Snapshot::new(buffer, point, &crate::config::Org::default()).is_err());
        assert!(buffer.history().is_empty());
    }
}
