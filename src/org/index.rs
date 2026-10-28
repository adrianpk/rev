mod limits;
mod scan;

pub use limits::{Limit, Limits};
pub use scan::Scan;

use crate::buffer::{Buffer, Error, Span, Stamp};
use crate::coordinates::Byte;

use super::{Source, SourceRange};

/// Coverage describes source indexing, not completion of semantic Org parsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    Pending,
    Complete,
    Incomplete(Limit),
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

    fn check(&self, buffer: &Buffer) -> Result<(), Error> {
        buffer.slice(&Span {
            stamp: self.stamp,
            range: 0..0,
        })?;
        Ok(())
    }
}
