//! Native table plans retain original cell source. Only the canonical buffer
//! transaction can publish edits or grant history and view authority.
mod action;
mod layout;
mod observe;

use super::{Cancellation, Coverage, Limits, Source, StructureKind as K, structure};
use crate::{
    buffer::{
        Buffer, Span, Stamp,
        edit::{Point, Replacement},
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
}
pub(crate) enum Output {
    Prompt,
    Plan(Plan),
    Fallback,
}
pub(crate) struct Plan {
    pub edits: Vec<Replacement>,
    pub cursor: Scalar,
}
struct Table {
    source: Range<usize>,
    rows: Vec<Row>,
    columns: usize,
    formulas: bool,
}
struct Row {
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
    pub fn new(buffer: &Buffer, point: Point) -> Result<Self, String> {
        buffer.region(point).map_err(|e| e.to_string())?;
        let text = Source::new(buffer).snapshot().map_err(|e| e.to_string())?;
        if text.len_bytes() > Limits::default().bytes {
            return Err("Table source exceeds native analysis byte limit".into());
        }
        Ok(Self {
            stamp: buffer.stamp(),
            point: text.char_to_byte(point.position.0),
            text,
            tab_width: buffer.policy().tab_width,
            newline: buffer.newline(),
        })
    }
    pub fn run(
        self,
        command: C,
        argument: Option<String>,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        let raw = structure::observe(self.text.clone(), Limits::default(), cancel);
        if raw.coverage != Coverage::Complete || raw.nodes.iter().any(|n| n.kind == K::Malformed) {
            return Err("Incomplete or malformed table source; nothing changed".into());
        }
        if command == C::TableCreate {
            return self.create(&raw.nodes, argument, cancel);
        }
        let Some(table) = Table::observe(&self.text, &raw.nodes, self.point, cancel)? else {
            return if command.table() && command != C::Return {
                Err("No native Org table at point".into())
            } else {
                Ok(Output::Fallback)
            };
        };
        if matches!(command, C::Return | C::TableNextRow)
            && table
                .rows
                .iter()
                .any(|r| self.point == r.body.start || self.point == r.body.end)
        {
            return Ok(Output::Fallback);
        }
        self.operate(table, command, cancel).map(Output::Plan)
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
        Ok(Plan { edits, cursor })
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
            let snapshot = Snapshot::new(buffer, point).unwrap();
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
        assert!(Snapshot::new(buffer, point).is_err());
        assert!(buffer.history().is_empty());
    }
}
