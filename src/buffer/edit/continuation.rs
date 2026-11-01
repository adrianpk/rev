use super::{Buffer, Change, Error, Operation, Prepared, Stamp, Transaction, insert};

impl Buffer {
    /// Continue one explicitly owned text-entry group. Only connected native
    /// replacements can join; unrelated edits keep their own history record.
    pub(crate) fn continue_edit(
        &self,
        prepared: &mut Prepared,
        first: Stamp,
    ) -> Result<bool, Error> {
        let Some(previous) = self.history.back().filter(|r| r.before == first) else {
            return Ok(false);
        };
        let Some(group) = previous.connected(&prepared.record)? else {
            return Ok(false);
        };
        prepared.group = Some(group);
        prepared.operation = Operation::Continued;
        Ok(true)
    }
}

impl Transaction {
    fn connected(&self, next: &Self) -> Result<Option<Self>, Error> {
        if !self.same_context(next) || self.permutation.is_some() || next.permutation.is_some() {
            return Ok(None);
        }
        let [current] = next.changes.as_slice() else {
            return Ok(None);
        };
        let mut group = self.clone();
        if !group.compose_change(current)? {
            return Ok(None);
        }
        group.after = next.after;
        group.after_state = next.after_state;
        group.after_points = next.after_points.clone();
        group.after_mark = next.after_mark;
        group.after_bookmarks = next.after_bookmarks.clone();
        group.after_locations = next.after_locations.clone();
        group.bytes = group.payload();
        Ok(Some(group))
    }
    fn prepend_change(&mut self, current: &Change) -> Result<bool, Error> {
        if self.changes.first().is_some_and(|first| {
            current.before.is_empty() && current.before.start < first.after.start
        }) {
            // A Visual block's empty top row may precede its first deletion.
            for change in &mut self.changes {
                change.after.start = shifted(change.after.start, current)?;
                change.after.end = shifted(change.after.end, current)?;
            }
            self.changes.insert(0, current.clone());
            return Ok(true);
        }
        Ok(false)
    }

    fn compose_change(&mut self, current: &Change) -> Result<bool, Error> {
        if self.prepend_change(current)? {
            return Ok(true);
        }
        let Some(index) = self
            .changes
            .iter()
            .position(|previous| previous.connects(current))
        else {
            return Ok(false);
        };
        if self
            .changes
            .get(index + 1)
            .is_some_and(|c| current.before.end >= c.after.start)
        {
            return Ok(false);
        }
        self.changes[index].compose(current)?;
        for change in &mut self.changes[index + 1..] {
            change.after.start = shifted(change.after.start, current)?;
            change.after.end = shifted(change.after.end, current)?;
        }
        Ok(true)
    }
}

impl Change {
    fn connects(&self, next: &Self) -> bool {
        next.before.start >= self.after.start && next.before.start <= self.after.end
    }

    fn compose(&mut self, next: &Self) -> Result<(), Error> {
        let start = next.before.start - self.after.start;
        let end = next.before.end.min(self.after.end) - self.after.start;
        self.inserted
            .try_remove(start..end)
            .map_err(|_| Error::Capacity)?;
        insert(&mut self.inserted, start, &next.inserted)?;
        if next.before.end > self.after.end {
            let extra = next.before.end - self.after.end;
            let from = next.removed.len_chars() - extra;
            let tail = next.removed.slice(from..).into();
            let at = self.removed.len_chars();
            insert(&mut self.removed, at, &tail)?;
            self.before.end = self.before.end.checked_add(extra).ok_or(Error::Capacity)?;
        }
        self.after.end = self
            .after
            .start
            .checked_add(self.inserted.len_chars())
            .ok_or(Error::Capacity)?;
        Ok(())
    }
}

fn shifted(position: usize, change: &Change) -> Result<usize, Error> {
    position
        .checked_sub(change.removed.len_chars())
        .and_then(|p| p.checked_add(change.inserted.len_chars()))
        .ok_or(Error::Capacity)
}
