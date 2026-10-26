mod mapping;

use std::collections::VecDeque;
use std::fmt;
use std::ops::Range;

use ropey::Rope;

use super::{Buffer, Span, Stamp};
use crate::coordinates::{self, Coordinates, Scalar};
use mapping::map_points;

/// Replacement ranges are ordered, disjoint scalar ranges in one BASE revision.
/// Equal starts are rejected, including multiple insertions at the same offset.
#[derive(Clone, Debug)]
pub struct Replacement {
    pub source: Span,
    pub text: String,
}

/// A view-owned point qualified by its currently displayed buffer revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub stamp: Stamp,
    pub position: Scalar,
}

/// Explicit initiating point in the final text; other points map independently.
#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    pub index: usize,
    pub position: Scalar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Source(super::Error),
    Coordinates(coordinates::Error),
    ReadOnly,
    Empty,
    Order,
    ChangedPositions,
    Cursor(usize),
    Capacity,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => error.fmt(formatter),
            Self::Coordinates(error) => error.fmt(formatter),
            Self::ReadOnly => formatter.write_str("buffer is read-only"),
            Self::Empty => formatter.write_str("edit has no replacements"),
            Self::Order => {
                formatter.write_str("base ranges must be ordered and disjoint with distinct starts")
            }
            Self::ChangedPositions => {
                formatter.write_str("points or mark changed after edit preparation")
            }
            Self::Cursor(index) => {
                write!(formatter, "initiating point index {index} is unavailable")
            }
            Self::Capacity => formatter.write_str("edit capacity or revision is exhausted"),
        }
    }
}

impl std::error::Error for Error {}

impl From<super::Error> for Error {
    fn from(error: super::Error) -> Self {
        Self::Source(error)
    }
}

impl From<coordinates::Error> for Error {
    fn from(error: coordinates::Error) -> Self {
        Self::Coordinates(error)
    }
}

/// Reversible fragments only; neither fragment is another authoritative document.
#[derive(Clone, Debug)]
pub struct Change {
    before: Range<usize>,
    after: Range<usize>,
    removed: Rope,
    inserted: Rope,
}

impl Change {
    pub fn before(&self) -> Range<usize> {
        self.before.clone()
    }

    pub fn after(&self) -> Range<usize> {
        self.after.clone()
    }

    pub fn removed(&self) -> &Rope {
        &self.removed
    }

    pub fn inserted(&self) -> &Rope {
        &self.inserted
    }

    fn payload(&self) -> usize {
        self.removed
            .len_bytes()
            .saturating_add(self.inserted.len_bytes())
    }
}

/// One published operation. Its stamps also invalidate revision-derived caches.
#[derive(Debug)]
pub struct Transaction {
    pub before: Stamp,
    pub after: Stamp,
    changes: Vec<Change>,
    before_points: Vec<Point>,
    after_points: Vec<Point>,
    pub before_mark: Option<Scalar>,
    pub after_mark: Option<Scalar>,
    bytes: usize,
}

impl Transaction {
    pub fn changes(&self) -> &[Change] {
        &self.changes
    }

    pub fn before_points(&self) -> &[Point] {
        &self.before_points
    }

    pub fn after_points(&self) -> &[Point] {
        &self.after_points
    }
}

/// A private copy-on-write candidate; dropping it cancels without publication.
/// Publication rechecks the live revision, read-only state, points, and mark.
pub struct Prepared {
    text: Rope,
    crlf: usize,
    record: Transaction,
}

impl Buffer {
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    pub fn mark(&self) -> Option<Scalar> {
        self.mark
    }

    pub fn set_mark(&mut self, mark: Option<Scalar>) -> Result<(), Error> {
        if let Some(position) = mark {
            mapping::boundary(&Coordinates::new(self)?, position)?;
        }
        self.mark = mark;
        Ok(())
    }

    pub fn history(&self) -> &VecDeque<Transaction> {
        &self.history
    }

    /// Validates source ranges and positions, then completes all fallible work
    /// on a private candidate. No text, history, point, or mark is published here.
    pub fn prepare(
        &self,
        edits: &[Replacement],
        points: &[Point],
        cursor: Option<Cursor>,
    ) -> Result<Prepared, Error> {
        self.admit(self.stamp())?;
        let after = self.next_stamp()?;
        let changes = self.changes(edits)?;
        let text = self.candidate(edits)?;
        let crlf = super::endings::update(&self.text, &text, &changes, self.crlf)?;
        let coordinates = Coordinates::for_text(text.slice(..), after);
        let after_points = map_points(self, &changes, points, &coordinates)?;
        let mut record = Transaction {
            before: self.stamp(),
            after,
            changes,
            before_points: points.to_vec(),
            after_points,
            before_mark: self.mark,
            after_mark: None,
            bytes: 0,
        };
        record.after_mark = mapping::map_mark(&record, &coordinates)?;
        record.bytes = record
            .changes
            .iter()
            .fold(0usize, |sum, change| sum.saturating_add(change.payload()));
        mapping::set_cursor(&mut record, cursor, &coordinates)?;
        Ok(Prepared { text, crlf, record })
    }

    /// The sole text publication path. The caller supplies every live point for
    /// this buffer; unchanged source snapshots are required before any mutation.
    pub fn publish(&mut self, prepared: Prepared, points: &mut [Point]) -> Result<(), Error> {
        self.admit(prepared.record.before)?;
        if points != prepared.record.before_points || self.mark != prepared.record.before_mark {
            return Err(Error::ChangedPositions);
        }
        self.history.try_reserve(1).map_err(|_| Error::Capacity)?;
        // From here onward there are no recoverable operations or callbacks.
        self.text = prepared.text;
        self.crlf = prepared.crlf;
        self.revision = prepared.record.after.revision;
        self.mark = prepared.record.after_mark;
        points.copy_from_slice(&prepared.record.after_points);
        self.history_bytes = self.history_bytes.saturating_add(prepared.record.bytes);
        self.history.push_back(prepared.record);
        self.prune();
        Ok(())
    }

    fn admit(&self, stamp: Stamp) -> Result<(), Error> {
        self.current(stamp)?;
        if self.read_only {
            return Err(Error::ReadOnly);
        }
        Ok(())
    }

    fn next_stamp(&self) -> Result<Stamp, Error> {
        let revision = self.revision.checked_add(1).ok_or(Error::Capacity)?;
        Ok(Stamp {
            buffer: self.id,
            revision,
        })
    }

    fn changes(&self, edits: &[Replacement]) -> Result<Vec<Change>, Error> {
        if edits.is_empty() {
            return Err(Error::Empty);
        }
        let mut changes = Vec::with_capacity(edits.len());
        let mut source = 0;
        let mut target: usize = 0;
        for edit in edits {
            let removed = self.slice(&edit.source)?;
            mapping::ordered(&changes, &edit.source.range)?;
            let start = target
                .checked_add(edit.source.range.start - source)
                .ok_or(Error::Capacity)?;
            let end = start
                .checked_add(edit.text.chars().count())
                .ok_or(Error::Capacity)?;
            changes.push(Change {
                before: edit.source.range.clone(),
                after: start..end,
                removed: removed.into(),
                inserted: Rope::from_str(&edit.text),
            });
            source = edit.source.range.end;
            target = end;
        }
        Ok(changes)
    }

    fn candidate(&self, edits: &[Replacement]) -> Result<Rope, Error> {
        let mut text = self.text.clone();
        // Descending application keeps every range in the admitted base space.
        for edit in edits.iter().rev() {
            text.try_remove(edit.source.range.clone())
                .map_err(|_| Error::Capacity)?;
            text.try_insert(edit.source.range.start, &edit.text)
                .map_err(|_| Error::Capacity)?;
        }
        Ok(text)
    }

    fn prune(&mut self) {
        // One transaction is one group until command grouping is introduced.
        // Always retain the newest reversible operation, even above the soft cap.
        while self.history.len() > 1
            && (self.history.len() > 512 || self.history_bytes > 64 * 1024 * 1024)
        {
            if let Some(record) = self.history.pop_front() {
                self.history_bytes = self.history_bytes.saturating_sub(record.bytes);
            }
        }
    }
}
