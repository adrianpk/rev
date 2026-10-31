use super::{Buffer, Cursor, Error, Operation, Point, Prepared, Replacement, Transaction, insert};

impl Buffer {
    /// Commands, view changes and save boundaries explicitly end ordinary typing.
    pub fn break_typing(&mut self) {
        self.typing = None;
    }

    /// Single ordinary scalar insertions may continue a checked typing group.
    /// Other replacements remain complete standalone operations.
    pub fn prepare_typing(
        &self,
        edit: &Replacement,
        points: &[Point],
        cursor: Cursor,
    ) -> Result<Prepared, Error> {
        let mut prepared = self.prepare(std::slice::from_ref(edit), points, Some(cursor))?;
        if !prepared.record.ordinary() {
            return Ok(prepared);
        }
        if let Some(previous) = self.typing_group(&prepared.record) {
            prepared.group = Some(previous.joined(&prepared.record)?);
        }
        prepared.operation = Operation::Typing;
        Ok(prepared)
    }

    fn typing_group(&self, next: &Transaction) -> Option<&Transaction> {
        if self.typing != Some(next.before) {
            return None;
        }
        let previous = self.history.back()?;
        previous.continues(next).then_some(previous)
    }

    pub(super) fn admit_group(&self, prepared: &Prepared) -> Result<(), Error> {
        if prepared.group.is_some() && self.typing != Some(prepared.record.before) {
            return Err(Error::ChangedPositions);
        }
        Ok(())
    }

    pub(super) fn record_typing(
        &mut self,
        record: Transaction,
        group: Option<Transaction>,
    ) -> Result<(), Error> {
        let record = match group {
            Some(group) => {
                let previous = self.history.pop_back().ok_or(Error::HistoryState)?;
                self.history_bytes = self.history_bytes.saturating_sub(previous.bytes);
                group
            }
            None => record,
        };
        self.record_edit(record);
        Ok(())
    }
}

impl Transaction {
    fn insertion(&self) -> Option<&super::Change> {
        match self.changes.as_slice() {
            [change] if change.before.is_empty() => Some(change),
            _ => None,
        }
    }

    fn ordinary(&self) -> bool {
        let Some(change) = self.insertion() else {
            return false;
        };
        let Some(index) = self.cursor else {
            return false;
        };
        let Some(before) = self.before_points.get(index) else {
            return false;
        };
        let Some(after) = self.after_points.get(index) else {
            return false;
        };
        before.position.0 == change.before.start
            && after.position.0 == change.after.end
            && change.inserted.len_chars() == 1
            && change
                .inserted
                .get_char(0)
                .is_some_and(|character| !character.is_control())
    }

    fn continues(&self, next: &Self) -> bool {
        let Some(previous) = self.insertion() else {
            return false;
        };
        let Some(current) = next.insertion() else {
            return false;
        };
        previous.after.end == current.before.start && self.same_context(next)
    }

    fn same_context(&self, next: &Self) -> bool {
        self.after_state == next.before_state
            && self.after_points == next.before_points
            && self.after_bookmarks == next.before_bookmarks
            && self.after_mark == next.before_mark
            && self.cursor == next.cursor
    }

    fn joined(&self, next: &Self) -> Result<Self, Error> {
        let mut group = self.clone();
        let previous = group.changes.first_mut().ok_or(Error::Order)?;
        let current = next.insertion().ok_or(Error::Order)?;
        let end = previous.inserted.len_chars();
        insert(&mut previous.inserted, end, &current.inserted)?;
        previous.after.end = current.after.end;
        group.after = next.after;
        group.after_state = next.after_state;
        group.after_points = next.after_points.clone();
        group.after_mark = next.after_mark;
        group.after_bookmarks = next.after_bookmarks.clone();
        group.bytes = group.payload();
        Ok(group)
    }
}
