//! Reserved native publication. All allocation, recoverable admission and
//! observations precede canonical text changes; installation itself cannot fail.
use super::{
    Buffer, Error, Operation, Point, Prepared, Stamp, State, Transaction, macro_lines,
    observe::Published,
};
use crate::{bookmark::Anchor, coordinates::Scalar, location::Snapshot};
use ropey::Rope;

pub(super) struct Publication {
    before: Stamp,
    after: Stamp,
    state: State,
    bom: bool,
    mark: Option<Scalar>,
    changed: usize,
    points: Vec<Point>,
    bookmarks: Vec<Anchor>,
    locations: Vec<Snapshot>,
    text: Rope,
    crlf: usize,
    next_state: u64,
    syntax: Vec<tree_sitter::InputEdit>,
    macro_lines: Vec<macro_lines::Mapped>,
    typing: Option<Stamp>,
    observation: Option<Published>,
}
/// A standalone edit with reserved history and mapped publication state.
/// Only the serialized native batch owner may consume it after final rechecks.
pub(crate) struct ReservedEdit {
    record: Transaction,
    publication: Publication,
    active: super::super::region::Mode,
}
impl ReservedEdit {
    pub(crate) fn after(&self) -> Stamp {
        self.record.after
    }
}
impl Publication {
    pub(super) fn prepare(
        buffer: &Buffer,
        prepared: Prepared,
    ) -> (Self, Transaction, Operation, Option<Transaction>) {
        let observation = buffer
            .observer
            .as_ref()
            .filter(|o| o.active())
            .map(|_| Published::capture(&buffer.text, &prepared.text, &prepared.record));
        let record = prepared.record;
        let publication = Self {
            before: record.before,
            after: record.after,
            state: record.after_state,
            bom: record.after_bom,
            mark: record.after_mark,
            changed: record
                .changes
                .first()
                .map_or(0, |change| change.before.start),
            points: record.after_points.clone(),
            bookmarks: record.after_bookmarks.clone(),
            locations: record.after_locations.clone(),
            text: prepared.text,
            crlf: prepared.crlf,
            next_state: prepared.next_state,
            syntax: prepared.syntax,
            macro_lines: prepared.macro_lines,
            typing: matches!(prepared.operation, Operation::Typing).then_some(record.after),
            observation,
        };
        (publication, record, prepared.operation, prepared.group)
    }
}
impl Buffer {
    /// Preserve the normal single-buffer path's checked history behavior while
    /// sharing the exact same canonical state installation with atomic batches.
    pub(super) fn install(
        &mut self,
        publication: Publication,
        points: &mut [Point],
    ) -> Option<Published> {
        self.syntax
            .edit(publication.before, publication.after, &publication.syntax);
        self.edit_org(publication.before, publication.after, &publication.syntax);
        self.markdown.observation = None;
        self.coordinates
            .lock()
            .expect("coordinate cache poisoned")
            .retain_prefix(
                publication.before,
                publication.after,
                Scalar(publication.changed),
            );
        self.text = publication.text;
        self.crlf = publication.crlf;
        self.revision = publication.after.revision;
        self.state = publication.state;
        self.bom = publication.bom;
        self.next_state = publication.next_state;
        self.mark = publication.mark;
        self.mark_mode = super::super::region::Mode::Inactive;
        points.copy_from_slice(&publication.points);
        self.bookmarks
            .lock()
            .expect("bookmark owner poisoned")
            .publish_anchors(publication.after, &publication.bookmarks);
        self.locations
            .lock()
            .expect("location owner poisoned")
            .publish(publication.after, &publication.locations);
        macro_lines::publish(publication.macro_lines, publication.after);
        self.typing = publication.typing;
        publication.observation
    }
    pub(crate) fn notify_publication(&self, observation: Option<Published>) {
        self.notify_event(crate::buffer::events::Kind::Changed);
        if let Some(observation) = observation
            && let Some(observer) = &self.observer
        {
            observer.published(observation);
        }
    }
    pub(crate) fn reserve_edit(
        &mut self,
        mut prepared: Prepared,
        points: &[Point],
    ) -> Result<ReservedEdit, Error> {
        if !matches!(prepared.operation, Operation::Edit)
            || prepared.group.is_some()
            || self.automatic.is_some()
        {
            return Err(Error::ChangedPositions);
        }
        // Package-generated edits have one independent native undo unit. They
        // cannot inherit an automatic iteration or a previous typing group.
        prepared.record.automatic = None;
        self.admit_publication(&prepared, points)?;
        let active = prepared.active;
        let (publication, record, _, _) = Publication::prepare(self, prepared);
        Ok(ReservedEdit {
            publication,
            record,
            active,
        })
    }
    pub(crate) fn check_reserved_edit(
        &self,
        edit: &ReservedEdit,
        points: &[Point],
    ) -> Result<(), Error> {
        self.admit(edit.record.before)?;
        self.check_positions(
            &edit.record,
            edit.active,
            &edit.publication.macro_lines,
            points,
        )?;
        if self.automatic.is_some() {
            return Err(Error::ChangedPositions);
        }
        Ok(())
    }
    /// The caller has rechecked every participant and holds exclusive native
    /// ownership. History capacity and all mapped state were reserved earlier.
    pub(crate) fn commit_reserved_edit(
        &mut self,
        edit: ReservedEdit,
        points: &mut [Point],
    ) -> Option<Published> {
        self.record_edit(edit.record);
        self.install(edit.publication, points)
    }
}
