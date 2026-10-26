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
    Typing,
    History(History),
}

impl History {
    pub fn name(self) -> &'static str {
        match self {
            Self::Undo => "undo",
            Self::Redo => "undo-redo",
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
        let prepared = self.prepare_history(direction, points, initiator)?;
        self.publish(prepared, points)
    }

    fn prepare_history(
        &self,
        direction: History,
        points: &[Point],
        initiator: Option<usize>,
    ) -> Result<Prepared, Error> {
        let record = self.history_record(direction, points.len(), initiator)?;
        let (_, target) = record.states(direction);
        let changes = record.replay(direction);
        let mut prepared = self.prepare_changes(changes, points, target)?;
        let coordinates = Coordinates::for_text(prepared.text.slice(..), prepared.record.after);
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
            Operation::Edit => self.record_edit(record),
            Operation::Typing => self.record_typing(record, group)?,
            Operation::History(direction) => self.transfer(direction)?,
        }
        Ok(())
    }

    pub(super) fn record_edit(&mut self, record: Transaction) {
        for discarded in self.redo.drain(..) {
            self.history_bytes = self.history_bytes.saturating_sub(discarded.bytes);
        }
        self.history_bytes = self.history_bytes.saturating_add(record.bytes);
        self.history.push_back(record);
        self.prune();
    }

    fn transfer(&mut self, direction: History) -> Result<(), Error> {
        let (source, target) = match direction {
            History::Undo => (&mut self.history, &mut self.redo),
            History::Redo => (&mut self.redo, &mut self.history),
        };
        let record = source.pop_back().ok_or(Error::History(direction))?;
        target.push_back(record);
        Ok(())
    }
}

impl Transaction {
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
