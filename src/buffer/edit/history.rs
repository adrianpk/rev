use std::collections::VecDeque;

use super::{Buffer, Change, Cursor, Error, Point, Prepared, Transaction, mapping};
use crate::buffer::State;
use crate::coordinates::Coordinates;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum History {
    Undo,
    Redo,
}

#[derive(Clone, Copy)]
pub(super) enum Operation {
    Edit,
    Reload,
    Typing,
    Continued,
    History(History),
}

impl History {
    pub fn name(self) -> &'static str {
        match self {
            Self::Undo => "undo",
            Self::Redo => "undo-redo",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Undo => "Undo a buffer-local group.",
            Self::Redo => "Reapply the next redo group.",
        }
    }

    pub(super) fn unavailable(self) -> &'static str {
        match self {
            Self::Undo => "Nothing to undo",
            Self::Redo => "Nothing to redo",
        }
    }
}

impl Buffer {
    pub fn redo_history(&self) -> &VecDeque<Transaction> {
        &self.redo
    }

    /// Replays one retained operation with a fresh revision. Supply every live
    /// point and the live initiating (or invoking) point's current array index.
    /// Other points map from their CURRENT positions, not historical snapshots.
    pub fn navigate(
        &mut self,
        direction: History,
        points: &mut [Point],
        initiator: Option<usize>,
    ) -> Result<(), Error> {
        let count = self.history_unit(direction).max(1);
        for _ in 0..count {
            let prepared = self.prepare_history(direction, points, initiator)?;
            self.publish(prepared, points)?;
        }
        Ok(())
    }

    pub(crate) fn prepare_history(
        &self,
        direction: History,
        points: &[Point],
        initiator: Option<usize>,
    ) -> Result<Prepared, Error> {
        let record = self.history_record(direction, points.len(), initiator)?;
        let (_, target) = record.states(direction);
        let changes = record.replay(direction);
        let permutation = record
            .permutation
            .as_ref()
            .map(|order| order.replay(direction));
        let mut prepared = self.prepare_changes(changes, points, target, permutation)?;
        prepared.record.after_bom = match direction {
            History::Undo => record.before_bom,
            History::Redo => record.after_bom,
        };
        let coordinates = Coordinates::for_text(
            prepared.text.slice(..),
            prepared.record.after,
            self.policy.tab_width,
        );
        prepared
            .record
            .restore_anchors(record, direction, &coordinates)?;
        prepared.record.bytes = prepared.record.payload();
        prepared.record.after_mark = record.mark(direction);
        if let Some(mark) = prepared.record.after_mark {
            mapping::boundary(&coordinates, mark)?;
        }
        let cursor = record.cursor(direction, initiator)?;
        mapping::set_cursor(&mut prepared.record, cursor, &coordinates)?;
        prepared.operation = Operation::History(direction);
        Ok(prepared)
    }

    fn history_record(
        &self,
        direction: History,
        point_count: usize,
        initiator: Option<usize>,
    ) -> Result<&Transaction, Error> {
        self.admit(self.stamp())?;
        if let Some(index) = initiator
            && index >= point_count
        {
            return Err(Error::Cursor(index));
        }
        let record = self
            .stack(direction)
            .back()
            .ok_or(Error::History(direction))?;
        let (source, _) = record.states(direction);
        if self.state != source {
            return Err(Error::HistoryState);
        }
        Ok(record)
    }

    fn stack(&self, direction: History) -> &VecDeque<Transaction> {
        match direction {
            History::Undo => &self.history,
            History::Redo => &self.redo,
        }
    }

    pub(super) fn reserve(&mut self, operation: Operation) -> Result<(), Error> {
        let target = match operation {
            Operation::History(History::Undo) => &mut self.redo,
            _ => &mut self.history,
        };
        target.try_reserve(1).map_err(|_| Error::Capacity)
    }

    pub(super) fn record(
        &mut self,
        record: Transaction,
        operation: Operation,
        group: Option<Transaction>,
    ) -> Result<(), Error> {
        match operation {
            Operation::Edit | Operation::Reload => self.record_edit(record),
            Operation::Typing | Operation::Continued => self.record_typing(record, group)?,
            Operation::History(direction) => self.transfer(direction)?,
        }
        Ok(())
    }

    pub(super) fn record_edit(&mut self, record: Transaction) {
        for discarded in self.redo.drain(..) {
            self.history_bytes = self.history_bytes.saturating_sub(discarded.bytes);
        }
        self.history_bytes = self.history_bytes.saturating_add(record.bytes);
        self.push_history(record);
        self.prune();
    }

    pub(super) fn push_history(&mut self, record: Transaction) {
        if record.automatic.is_none()
            || self
                .history
                .back()
                .is_none_or(|last| !last.same_automatic(&record))
        {
            self.history_units += 1;
        }
        self.history.push_back(record);
    }

    pub(super) fn pop_history(&mut self) -> Option<Transaction> {
        let record = self.history.pop_back()?;
        if record.automatic.is_none()
            || self
                .history
                .back()
                .is_none_or(|last| !last.same_automatic(&record))
        {
            self.history_units -= 1;
        }
        Some(record)
    }

    fn transfer(&mut self, direction: History) -> Result<(), Error> {
        match direction {
            History::Undo => {
                let record = self.pop_history().ok_or(Error::History(direction))?;
                self.redo.push_back(record);
            }
            History::Redo => {
                let record = self.redo.pop_back().ok_or(Error::History(direction))?;
                self.push_history(record);
            }
        }
        Ok(())
    }
}

impl Transaction {
    fn restore_anchors(
        &mut self,
        record: &Self,
        direction: History,
        coordinates: &Coordinates<'_>,
    ) -> Result<(), Error> {
        super::bookmarks::restore(
            &mut self.after_bookmarks,
            match direction {
                History::Undo => &record.before_bookmarks,
                History::Redo => &record.after_bookmarks,
            },
            coordinates,
        )?;
        super::locations::restore(
            &mut self.after_locations,
            match direction {
                History::Undo => &record.before_locations,
                History::Redo => &record.after_locations,
            },
            coordinates,
        )?;
        Ok(())
    }

    fn states(&self, direction: History) -> (State, State) {
        match direction {
            History::Undo => (self.after_state, self.before_state),
            History::Redo => (self.before_state, self.after_state),
        }
    }

    fn replay(&self, direction: History) -> Vec<Change> {
        self.changes
            .iter()
            .map(|change| match direction {
                History::Redo => change.clone(),
                History::Undo => Change {
                    before: change.after.clone(),
                    after: change.before.clone(),
                    removed: change.inserted.clone(),
                    inserted: change.removed.clone(),
                },
            })
            .collect()
    }

    fn mark(&self, direction: History) -> Option<crate::coordinates::Scalar> {
        match direction {
            History::Undo => self.before_mark,
            History::Redo => self.after_mark,
        }
    }

    fn cursor(
        &self,
        direction: History,
        initiator: Option<usize>,
    ) -> Result<Option<Cursor>, Error> {
        let (Some(original), Some(index)) = (self.cursor, initiator) else {
            return Ok(None);
        };
        let points = match direction {
            History::Undo => &self.before_points,
            History::Redo => &self.after_points,
        };
        let position = points
            .get(original)
            .ok_or(Error::Cursor(original))?
            .position;
        Ok(Some(Cursor { index, position }))
    }
}
