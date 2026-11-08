mod automatic;
mod bookmarks;
mod continuation;
mod grouping;
pub(crate) use automatic::AutomaticGroup;
mod history;
mod locations;
mod mapping;
mod observe;
pub(crate) use observe::{Observer, Published};
mod reload;
mod reorder;
mod retained;
mod syntax;
mod views;
use reorder::Permutation;

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

/// An unchanged source fragment and its scalar range in the private candidate.
/// The canonical edit owner verifies byte equality and disjoint ranges before
/// using it to retain shared points and marks through a structural edit.
#[derive(Clone, Debug)]
pub(crate) struct Retained {
    pub source: Span,
    pub output: Range<usize>,
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
    before_bom: bool,
    after_bom: bool,
    changes: Vec<Change>,
    before_bookmarks: Vec<crate::bookmark::Anchor>,
    after_bookmarks: Vec<crate::bookmark::Anchor>,
    before_locations: Vec<crate::location::Snapshot>,
    after_locations: Vec<crate::location::Snapshot>,
    before_points: Vec<Point>,
    after_points: Vec<Point>,
    pub before_mark: Option<Scalar>,
    pub after_mark: Option<Scalar>,
    bytes: usize,
    cursor: Option<usize>,
    origin: Option<crate::window::ViewId>,
    permutation: Option<Permutation>,
    automatic: Option<std::sync::Arc<AutomaticGroup>>,
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
    syntax: Vec<tree_sitter::InputEdit>,
    text: Rope,
    crlf: usize,
    record: Transaction,
    operation: Operation,
    next_state: u64,
    group: Option<Transaction>,
    active: super::region::Mode,
    tab_width: usize,
}

impl Buffer {
    /// Bound transient input before a native caller materializes a transfer.
    /// Final replacement size and complete batch limits are checked below.
    pub(crate) fn check_edit_payload(&self, bytes: usize) -> Result<(), Error> {
        if self.text_byte_limit.is_some_and(|limit| bytes > limit) {
            return Err(Error::Capacity);
        }
        Ok(())
    }

    pub(crate) fn edit_payload_limit(&self) -> Option<usize> {
        self.text_byte_limit
    }

    pub fn is_readonly(&self) -> bool {
        self.read_only || self.terminal().is_some()
    }

    pub fn set_readonly(&mut self, read_only: bool) {
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
        self.mark_mode = super::region::Mode::Inactive;
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
        self.prepare_ordered(edits, points, cursor, None)
    }

    fn prepare_ordered(
        &self,
        edits: &[Replacement],
        points: &[Point],
        cursor: Option<Cursor>,
        permutation: Option<Permutation>,
    ) -> Result<Prepared, Error> {
        self.admit(self.stamp())?;
        if let Some(limit) = self.text_byte_limit {
            if edits.len() > 64 || points.len() > 16 {
                return Err(Error::Capacity);
            }
            let mut bytes = self.text.len_bytes();
            for edit in edits {
                bytes = bytes
                    .checked_sub(self.slice(&edit.source)?.len_bytes())
                    .and_then(|n| n.checked_add(edit.text.len()))
                    .ok_or(Error::Capacity)?;
            }
            if bytes > limit {
                return Err(Error::Capacity);
            }
        }
        let next_state = self.next_state.checked_add(1).ok_or(Error::Capacity)?;
        let changes = self.changes(edits)?;
        let state = State {
            buffer: self.id,
            sequence: self.next_state,
        };
        let mut prepared = self.prepare_changes(changes, points, state, permutation)?;
        let coordinates = Coordinates::for_text(
            prepared.text.slice(..),
            prepared.record.after,
            self.policy.tab_width,
        );
        mapping::set_cursor(&mut prepared.record, cursor, &coordinates)?;
        prepared.next_state = next_state;
        Ok(prepared)
    }

    fn prepare_changes(
        &self,
        changes: Vec<Change>,
        points: &[Point],
        state: State,
        permutation: Option<Permutation>,
    ) -> Result<Prepared, Error> {
        let after = self.next_stamp()?;
        let text = self.candidate(&changes)?;
        let crlf = super::endings::update(&self.text, &text, &changes, self.crlf)?;
        let coordinates = Coordinates::for_text(text.slice(..), after, self.policy.tab_width);
        let after_points = map_points(self, &changes, points, &coordinates, permutation.as_ref())?;
        let mut record = Transaction {
            before: self.stamp(),
            after,
            before_state: self.state,
            after_state: state,
            before_bom: self.bom,
            after_bom: self.bom,
            before_bookmarks: self
                .bookmarks
                .lock()
                .expect("bookmark owner poisoned")
                .anchors(self.stamp())?,
            after_bookmarks: Vec::new(),
            before_locations: Vec::new(),
            after_locations: Vec::new(),
            changes,
            before_points: points.to_vec(),
            after_points,
            before_mark: self.mark,
            after_mark: None,
            bytes: 0,
            cursor: None,
            origin: None,
            permutation,
            automatic: self.automatic.clone(),
        };
        record.after_mark = mapping::map_mark(&record, &coordinates)?;
        record.after_bookmarks = bookmarks::map(&record, &coordinates)?;
        self.prepare_location_anchors(&mut record, &coordinates)?;
        record.bytes = record.payload();
        let syntax = syntax::edits(&self.text, &record.changes)?;
        Ok(Prepared {
            syntax,
            text,
            crlf,
            record,
            operation: Operation::Edit,
            next_state: self.next_state,
            group: None,
            active: self.mark_mode,
            tab_width: self.policy.tab_width,
        })
    }

    fn prepare_location_anchors(
        &self,
        record: &mut Transaction,
        coordinates: &Coordinates<'_>,
    ) -> Result<(), Error> {
        record.before_locations = self
            .locations
            .lock()
            .expect("location owner poisoned")
            .anchors(self.stamp())?;
        record.after_locations = locations::map(record, coordinates)?;
        Ok(())
    }

    /// The sole text publication path. The caller supplies every live point for
    /// this buffer; unchanged source snapshots are required before any mutation.
    pub fn publish(&mut self, prepared: Prepared, points: &mut [Point]) -> Result<(), Error> {
        self.admit_publication(&prepared, points)?;
        if let Some(group) = &self.automatic {
            group.charge(
                prepared
                    .record
                    .changes
                    .iter()
                    .map(|change| change.inserted.len_bytes())
                    .sum(),
            )?;
        }
        let observation = self
            .observer
            .as_ref()
            .filter(|observer| observer.active())
            .map(|_| Published::capture(&self.text, &prepared.text, &prepared.record));
        // Queue admission completes before any text or live position changes.
        let before = prepared.record.before;
        let after = prepared.record.after;
        let state = prepared.record.after_state;
        let bom = prepared.record.after_bom;
        let mark = prepared.record.after_mark;
        let after_points = prepared.record.after_points.clone();
        let after_bookmarks = prepared.record.after_bookmarks.clone();
        let after_locations = prepared.record.after_locations.clone();
        let changed = prepared
            .record
            .changes
            .first()
            .map_or(0, |change| change.before.start);
        self.record(prepared.record, prepared.operation, prepared.group)?;
        // Canonical state updates below are infallible. Optional observations run
        // only after every canonical text, point and anchor has been published.
        self.syntax.edit(before, after, &prepared.syntax);
        self.edit_org(before, after, &prepared.syntax);
        self.coordinates
            .lock()
            .expect("coordinate cache poisoned")
            .retain_prefix(before, after, crate::coordinates::Scalar(changed));
        self.text = prepared.text;
        self.crlf = prepared.crlf;
        self.revision = after.revision;
        self.state = state;
        self.bom = bom;
        self.next_state = prepared.next_state;
        self.mark = mark;
        self.mark_mode = super::region::Mode::Inactive;
        points.copy_from_slice(&after_points);
        self.bookmarks
            .lock()
            .expect("bookmark owner poisoned")
            .publish_anchors(after, &after_bookmarks);
        self.locations
            .lock()
            .expect("location owner poisoned")
            .publish(after, &after_locations);
        self.typing = match prepared.operation {
            Operation::Typing => Some(after),
            _ => None,
        };
        // Optional observation follows the complete, irreversible publication.
        if let Some(observation) = observation
            && let Some(observer) = &self.observer
        {
            observer.published(observation);
        }
        Ok(())
    }

    /// Reserve every recoverable publication requirement before a synchronous
    /// multi-buffer operation commits any participant. No text is changed.
    pub(crate) fn admit_publication(
        &mut self,
        prepared: &Prepared,
        points: &[Point],
    ) -> Result<(), Error> {
        if matches!(prepared.operation, Operation::Reload) {
            self.current(prepared.record.before)?;
        } else {
            self.admit(prepared.record.before)?;
        }
        if points != prepared.record.before_points
            || self.mark != prepared.record.before_mark
            || self.mark_mode != prepared.active
        {
            return Err(Error::ChangedPositions);
        }
        self.admit_anchors(prepared)?;
        self.admit_group(prepared)?;
        self.reserve(prepared.operation)?;
        Ok(())
    }

    fn admit_anchors(&self, prepared: &Prepared) -> Result<(), Error> {
        if self
            .bookmarks
            .lock()
            .expect("bookmark owner poisoned")
            .anchors(self.stamp())?
            != prepared.record.before_bookmarks
        {
            return Err(Error::ChangedPositions);
        }
        if self
            .locations
            .lock()
            .expect("location owner poisoned")
            .anchors(self.stamp())?
            != prepared.record.before_locations
        {
            return Err(Error::ChangedPositions);
        }
        Ok(())
    }

    fn admit(&self, stamp: Stamp) -> Result<(), Error> {
        self.current(stamp)?;
        if self.is_readonly() {
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
        // An automatic iteration is indivisible even when it contains several
        // native records. Retain its primitive mappings rather than flattening
        // unchanged source ranges into a replacement.
        while self.history_units > 1
            && (self.history_units > self.history_policy.group_limit
                || self.history_bytes > self.history_policy.payload_target_bytes)
        {
            self.history_units -= 1;
            let first = self.history.pop_front().expect("retained history unit");
            self.history_bytes = self.history_bytes.saturating_sub(first.bytes);
            if first.automatic.is_some() {
                while self
                    .history
                    .front()
                    .is_some_and(|next| first.same_automatic(next))
                {
                    let removed = self.history.pop_front().expect("same unit");
                    self.history_bytes = self.history_bytes.saturating_sub(removed.bytes);
                }
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

impl Prepared {
    /// Admit a resulting mark on the private candidate, preserving the original
    /// mark for native undo and applying normal grapheme endpoint adjustment.
    pub(crate) fn set_mark(&mut self, mark: Scalar) -> Result<(), Error> {
        let coordinates =
            Coordinates::for_text(self.text.slice(..), self.record.after, self.tab_width);
        let mark = if coordinates.is_boundary(mark)? {
            mark
        } else {
            coordinates.next(mark)?
        };
        self.record.after_mark = Some(mark);
        Ok(())
    }

    pub(in crate::buffer) fn saved_output(&self) -> (&Rope, Stamp, State) {
        (&self.text, self.record.after, self.record.after_state)
    }
}

impl Transaction {
    fn payload(&self) -> usize {
        let text = self
            .changes
            .iter()
            .fold(0usize, |sum, change| sum.saturating_add(change.payload()));
        let text = text.saturating_add(
            (self.before_locations.len() + self.after_locations.len())
                .saturating_mul(std::mem::size_of::<crate::location::Snapshot>()),
        );
        text.saturating_add(
            (self.before_bookmarks.len() + self.after_bookmarks.len())
                .saturating_mul(std::mem::size_of::<crate::bookmark::Anchor>()),
        )
    }
}
