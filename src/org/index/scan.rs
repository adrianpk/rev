use std::time::{Duration, Instant};

use ropey::Rope;

use crate::buffer::{Buffer, Error};

use super::{Coverage, Index, Limit, Limits, Position, Row};
use crate::org::{Cancellation, Source};

const STEP_BYTES: usize = 4096;
const STEP_TIME: Duration = Duration::from_millis(10);

/// Owned immutable rope observation with bounded compute steps. No background
/// work starts at construction, and no full-document string is allocated.
pub struct Scan {
    text: Rope,
    index: Index,
    position: Position,
    line_start: Position,
    prefix: Prefix,
    ancestors: Vec<(usize, usize)>,
}

#[derive(Default)]
enum Prefix {
    #[default]
    Start,
    Stars(usize),
    Heading(usize),
    CarriageReturn(usize),
    Text,
}

impl Prefix {
    fn push(&mut self, character: char) {
        *self = match (&self, character) {
            (Self::Start, '*') => Self::Stars(1),
            (Self::Stars(level), '*') => Self::Stars(level + 1),
            (Self::Stars(level), ' ' | '\t') => Self::Heading(*level),
            (Self::Stars(level), '\r') => Self::CarriageReturn(*level),
            (Self::Heading(level), _) => Self::Heading(*level),
            _ => Self::Text,
        };
    }

    fn level(&self) -> Option<usize> {
        match self {
            Self::Stars(level) | Self::Heading(level) | Self::CarriageReturn(level) => Some(*level),
            _ => None,
        }
    }
}

impl Scan {
    pub fn new(buffer: &Buffer, limits: Limits) -> Result<Self, Error> {
        Ok(Self {
            text: Source::new(buffer).snapshot()?,
            index: Index {
                stamp: buffer.stamp(),
                limits,
                rows: Vec::new(),
                covered: Position::default(),
                coverage: Coverage::Pending,
            },
            position: Position::default(),
            line_start: Position::default(),
            prefix: Prefix::default(),
            ancestors: Vec::new(),
        })
    }

    /// Move the retained prefix into the next observation; no row vector or
    /// document is copied on the input path. Old worker results cannot seed it.
    pub fn resume(index: Index, buffer: &Buffer) -> Result<Self, Error> {
        index.check(buffer)?;
        let mut ancestors = Vec::new();
        let mut heading = index.rows.len().checked_sub(1);
        while let Some(row) = heading {
            let line = &index.rows[row];
            if let Some(level) = line.level {
                ancestors.push((level, row));
            }
            heading = line.parent;
        }
        ancestors.reverse();
        Ok(Self {
            text: Source::new(buffer).snapshot()?,
            position: index.covered,
            line_start: index.covered,
            index,
            prefix: Prefix::default(),
            ancestors,
        })
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn scanned_bytes(&self) -> crate::coordinates::Byte {
        crate::coordinates::Byte(self.position.byte)
    }

    /// Preserve the actual coverage state when stopping before completion.
    pub fn into_index(self) -> Index {
        self.index
    }

    /// Process at most 4096 source bytes or 10ms before returning to the owner.
    /// A single scalar observation completes before its deadline is rechecked.
    pub fn advance(&mut self, cancellation: &Cancellation) -> Coverage {
        if self.index.coverage != Coverage::Pending {
            return self.index.coverage;
        }
        if self.cancelled(cancellation) {
            return Coverage::Cancelled;
        }
        let started = Instant::now();
        let start_byte = self.position.byte;
        while self.position.scalar < self.text.len_chars() {
            if self.cancelled(cancellation) {
                return Coverage::Cancelled;
            }
            let character = self.text.char(self.position.scalar);
            if self.yielded(started, start_byte, character) {
                return Coverage::Pending;
            }
            if !self.next(character) {
                return self.index.coverage;
            }
        }
        if self.finish_line() {
            self.index.coverage = Coverage::Complete;
        }
        self.index.coverage
    }

    pub(in crate::org) fn run(mut self, cancellation: &Cancellation) -> Index {
        while self.advance(cancellation) == Coverage::Pending {
            std::thread::yield_now();
        }
        self.into_index()
    }

    fn cancelled(&mut self, cancellation: &Cancellation) -> bool {
        if cancellation.is_cancelled() {
            self.index.coverage = Coverage::Cancelled;
            return true;
        }
        false
    }

    fn yielded(&self, started: Instant, start_byte: usize, character: char) -> bool {
        let worked = self.position.byte - start_byte;
        worked + character.len_utf8() > STEP_BYTES || (worked > 0 && started.elapsed() >= STEP_TIME)
    }

    fn next(&mut self, character: char) -> bool {
        if self.position.byte + character.len_utf8() > self.index.limits.bytes {
            self.index.coverage = Coverage::Incomplete(Limit::AnalysisBytes);
            return false;
        }
        self.position.scalar += 1;
        self.position.byte += character.len_utf8();
        if character == '\n' {
            return self.finish_line();
        }
        self.prefix.push(character);
        true
    }

    fn finish_line(&mut self) -> bool {
        let level = self.prefix.level();
        if self.index.rows.len() == self.index.limits.nodes {
            self.index.coverage = Coverage::Incomplete(Limit::SourceNodes);
            return false;
        }
        if level.is_some_and(|level| level > self.index.limits.depth) {
            self.index.coverage = Coverage::Incomplete(Limit::SourceDepth);
            return false;
        }
        self.push_line(level);
        self.line_start = self.position;
        self.index.covered = self.position;
        self.prefix = Prefix::default();
        true
    }

    fn push_line(&mut self, level: Option<usize>) {
        if let Some(level) = level {
            while self
                .ancestors
                .last()
                .is_some_and(|(depth, _)| *depth >= level)
            {
                self.ancestors.pop();
            }
        }
        let parent = self.ancestors.last().map(|(_, row)| *row);
        let row = self.index.rows.len();
        self.index.rows.push(Row {
            start: self.line_start,
            end: self.position,
            level,
            parent,
        });
        if let Some(level) = level {
            self.ancestors.push((level, row));
        }
    }
}
