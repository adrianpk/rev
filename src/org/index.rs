mod limits;
mod scan;

pub use limits::{Limit, Limits};
pub use scan::Scan;

use crate::buffer::{Buffer, Error, Stamp};
use crate::coordinates::Byte;

use super::{Source, SourceRange};

/// Coverage describes source indexing, not completion of semantic Org parsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    Pending,
    Complete,
    Incomplete(Limit),
    Cancelled,
}

/// One original logical line and its outline-prefix ancestry. A star prefix is
/// a structural candidate; syntax recognition supplies literal/block contexts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub source: SourceRange,
    pub bytes: std::ops::Range<Byte>,
    pub outline_level: Option<usize>,
    pub parent: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Position {
    scalar: usize,
    byte: usize,
}

struct Row {
    start: Position,
    end: Position,
    level: Option<usize>,
    parent: Option<usize>,
}

/// Bounded derived offsets only. The buffer remains the sole editable source.
pub struct Index {
    stamp: Stamp,
    limits: Limits,
    rows: Vec<Row>,
    covered: Position,
    coverage: Coverage,
}

impl Index {
    pub fn stamp(&self) -> Stamp {
        self.stamp
    }

    pub fn coverage(&self) -> Coverage {
        self.coverage
    }

    /// Only complete logical lines count as covered, even after a partial scan.
    pub fn covered_bytes(&self) -> Byte {
        Byte(self.covered.byte)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Validate the current document before qualifying a row for source access.
    pub fn line(&self, buffer: &Buffer, row: usize) -> Result<Line, Error> {
        self.check(buffer)?;
        let line = self.rows.get(row).ok_or(Error::Line {
            index: row,
            len: self.rows.len(),
        })?;
        let bytes = Byte(line.start.byte)..Byte(line.end.byte);
        Ok(Line {
            source: Source::new(buffer).range(bytes.clone())?,
            bytes,
            outline_level: line.level,
            parent: line.parent,
        })
    }

    pub(crate) fn check(&self, buffer: &Buffer) -> Result<(), Error> {
        super::source::current(buffer, self.stamp)
    }

    /// Canonical publication supplies this exact mutation, independently of
    /// typing-history grouping. Unaffected prefix offsets keep their identity.
    pub(crate) fn edited(&mut self, before: Stamp, after: Stamp, edits: &[tree_sitter::InputEdit]) {
        assert_eq!(
            self.stamp, before,
            "installed Org index matches canonical text"
        );
        let first = edits.iter().map(|edit| edit.start_byte).min().unwrap_or(0);
        let scope = self.scope(first.saturating_sub(1));
        self.covered = self
            .rows
            .get(scope)
            .map(|row| row.start)
            .unwrap_or(self.covered);
        self.rows.truncate(scope);
        self.stamp = after;
        self.coverage = Coverage::Pending;
    }

    fn scope(&self, byte: usize) -> usize {
        let row = self
            .rows
            .partition_point(|row| row.end.byte <= byte && row.start.byte < row.end.byte);
        let Some(line) = self.rows.get(row) else {
            return row;
        };
        if line.level.is_some() {
            return row;
        }
        line.parent.unwrap_or(row)
    }
}
