mod grouping;
mod history;
mod mapping;
mod views;

pub use history::History;
use history::Operation;

use std::collections::VecDeque;
use std::fmt;
use std::ops::Range;

use ropey::Rope;

use super::{Buffer, Span, Stamp, State};
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
    History(History),
    HistoryState,
    Mark,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => error.fmt(formatter),
            Self::Coordinates(error) => error.fmt(formatter),
            Self::Cursor(index) => {
                write!(formatter, "initiating point index {index} is unavailable")
            }
            Self::History(direction) => formatter.write_str(direction.unavailable()),
            _ => formatter.write_str(self.message()),
        }
    }
}

impl Error {
    fn message(self) -> &'static str {
        match self {
            Self::ReadOnly => "buffer is read-only",
            Self::Empty => "edit has no replacements",
            Self::Order => "base ranges must be ordered and disjoint with distinct starts",
            Self::ChangedPositions => {
                "points, mark, selection or typing continuity changed after edit preparation"
            }
            Self::Capacity => "edit capacity or revision is exhausted",
            Self::HistoryState => "history does not match the logical text state",
            Self::Mark => "Mark is not set",
            _ => unreachable!("parameterized errors format their own context"),
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
#[derive(Clone, Debug)]
pub struct Transaction {
    pub before: Stamp,
    pub after: Stamp,
    pub before_state: State,
    pub after_state: State,
    changes: Vec<Change>,
    before_points: Vec<Point>,
    after_points: Vec<Point>,
    pub before_mark: Option<Scalar>,
    pub after_mark: Option<Scalar>,
    bytes: usize,
    cursor: Option<usize>,
    origin: Option<crate::window::ViewId>,
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
    operation: Operation,
    next_state: u64,
    group: Option<Transaction>,
    active: bool,
}

impl Buffer {
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
        self.break_typing();
    }

    pub fn mark(&self) -> Option<Scalar> {
        self.mark
    }

    pub fn set_mark(&mut self, mark: Option<Scalar>) -> Result<(), Error> {
        if let Some(position) = mark {
            mapping::boundary(&Coordinates::new(self)?, position)?;
        }
        self.mark = mark;
        self.mark_active = false;
        self.break_typing();
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
        let next_state = self.next_state.checked_add(1).ok_or(Error::Capacity)?;
        let changes = self.changes(edits)?;
        let state = State {
            buffer: self.id,
            sequence: self.next_state,
        };
        let mut prepared = self.prepare_changes(changes, points, state)?;
        let coordinates = Coordinates::for_text(prepared.text.slice(..), prepared.record.after);
        mapping::set_cursor(&mut prepared.record, cursor, &coordinates)?;
        prepared.next_state = next_state;
        Ok(prepared)
    }

    fn prepare_changes(
        &self,
        changes: Vec<Change>,
        points: &[Point],
        state: State,
    ) -> Result<Prepared, Error> {
        let after = self.next_stamp()?;
        let text = self.candidate(&changes)?;
        let crlf = super::endings::update(&self.text, &text, &changes, self.crlf)?;
        let coordinates = Coordinates::for_text(text.slice(..), after);
        let after_points = map_points(self, &changes, points, &coordinates)?;
        let mut record = Transaction {
            before: self.stamp(),
            after,
            before_state: self.state,
            after_state: state,
            changes,
            before_points: points.to_vec(),
            after_points,
            before_mark: self.mark,
            after_mark: None,
            bytes: 0,
            cursor: None,
            origin: None,
        };
        record.after_mark = mapping::map_mark(&record, &coordinates)?;
        record.bytes = record
            .changes
            .iter()
            .fold(0usize, |sum, change| sum.saturating_add(change.payload()));
        Ok(Prepared {
            text,
            crlf,
            record,
            operation: Operation::Edit,
            next_state: self.next_state,
            group: None,
            active: self.mark_active,
        })
    }

    /// The sole text publication path. The caller supplies every live point for
    /// this buffer; unchanged source snapshots are required before any mutation.
    pub fn publish(&mut self, prepared: Prepared, points: &mut [Point]) -> Result<(), Error> {
        self.admit(prepared.record.before)?;
        if points != prepared.record.before_points
            || self.mark != prepared.record.before_mark
            || self.mark_active != prepared.active
        {
            return Err(Error::ChangedPositions);
        }
        self.admit_group(&prepared)?;
        self.reserve(prepared.operation)?;
        // Queue admission completes before any text or live position changes.
        let after = prepared.record.after;
        let state = prepared.record.after_state;
        let mark = prepared.record.after_mark;
        let after_points = prepared.record.after_points.clone();
        self.record(prepared.record, prepared.operation, prepared.group)?;
        // From here onward there are no recoverable operations or callbacks.
        self.text = prepared.text;
        self.crlf = prepared.crlf;
        self.revision = after.revision;
        self.state = state;
        self.next_state = prepared.next_state;
        self.mark = mark;
        self.mark_active = false;
        points.copy_from_slice(&after_points);
        self.typing = match prepared.operation {
            Operation::Typing => Some(after),
            _ => None,
        };
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

    fn candidate(&self, changes: &[Change]) -> Result<Rope, Error> {
        let mut text = self.text.clone();
        // Descending application keeps every range in the admitted base space.
        for change in changes.iter().rev() {
            text.try_remove(change.before.clone())
                .map_err(|_| Error::Capacity)?;
            insert(&mut text, change.before.start, &change.inserted)?;
        }
        Ok(text)
    }

    fn prune(&mut self) {
        // Prune whole groups only, always retaining the newest coherent group
        // even when its payload alone exceeds the soft target.
        while self.history.len() > 1
            && (self.history.len() > 512 || self.history_bytes > 64 * 1024 * 1024)
        {
            if let Some(record) = self.history.pop_front() {
                self.history_bytes = self.history_bytes.saturating_sub(record.bytes);
            }
        }
    }
}

fn insert(text: &mut Rope, mut position: usize, fragment: &Rope) -> Result<(), Error> {
    for chunk in fragment.chunks() {
        text.try_insert(position, chunk)
            .map_err(|_| Error::Capacity)?;
        position = position
            .checked_add(chunk.chars().count())
            .ok_or(Error::Capacity)?;
    }
    Ok(())
}
