use super::Transaction;
use crate::buffer::edit::Change;
use ropey::{Rope, RopeSlice};

pub(super) struct Mutation {
    pub(super) row: usize,
    pub(super) start: usize,
    pub(super) delta: isize,
}

struct Budget<'a> {
    remaining: &'a mut usize,
}

impl Budget<'_> {
    fn charge(&mut self, bytes: usize) -> Option<()> {
        *self.remaining = self.remaining.checked_sub(bytes)?;
        Some(())
    }

    fn fragment(&mut self, text: &Rope) -> Option<()> {
        self.charge(text.len_bytes())?;
        (!text.chars().any(|ch| ch == '\r' || ch == '\n')).then_some(())
    }

    fn line<'a>(&mut self, text: &'a Rope, row: usize) -> Option<RopeSlice<'a>> {
        let line = text.get_line(row)?;
        self.charge(line.len_bytes())?;
        Some(line)
    }

    fn same_line(&mut self, before: &Rope, after: &Rope, change: &Change) -> Option<usize> {
        self.fragment(change.removed())?;
        self.fragment(change.inserted())?;
        let row = before.try_char_to_line(change.before().start).ok()?;
        if row != after.try_char_to_line(change.after().start).ok()? {
            return None;
        }
        // Reserve the complete before/after lines before format inspection.
        // Never materialize an unchecked full logical line here.
        self.line(before, row)?;
        self.line(after, row)?;
        Some(row)
    }
}

impl Mutation {
    pub(super) fn inspect(
        before: &Rope,
        after: &Rope,
        record: &Transaction,
        remaining: &mut usize,
    ) -> Option<Self> {
        let [change] = record.changes() else {
            return None;
        };
        if before.len_lines() != after.len_lines() {
            return None;
        }
        let mut budget = Budget { remaining };
        let row = budget.same_line(before, after, change)?;
        let delta = isize::try_from(change.inserted().len_bytes())
            .ok()?
            .checked_sub(isize::try_from(change.removed().len_bytes()).ok()?)?;
        Some(Self {
            row,
            start: before.try_line_to_byte(row).ok()?,
            delta,
        })
    }
}
