//! Checked mapping foundation for display carry. This layer proves only source
//! shape and coordinates; format owners must still admit structure before any
//! display consumer can use it. Exact action observations remain unchanged.

mod inspection;
#[cfg(test)]
mod tests;

use super::{Buffer, Stamp, edit::Transaction};
use inspection::Mutation;
use ropey::Rope;

const ROW_LIMIT: usize = 8;
const SCAN_BYTES: usize = 32 * 1024;

/// Fixed storage, independent of the number of publications since the base.
/// Every recorded row is entirely lexically dirty, including zero-net edits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Mapping {
    base: Stamp,
    current: Stamp,
    base_bytes: usize,
    bytes: usize,
    rows: [Option<Row>; ROW_LIMIT],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Row {
    index: usize,
    base_start: usize,
    delta: isize,
}

impl Mapping {
    fn exact(stamp: Stamp, bytes: usize) -> Self {
        Self {
            base: stamp,
            current: stamp,
            base_bytes: bytes,
            bytes,
            rows: [None; ROW_LIMIT],
        }
    }

    fn prepare(self, before: &Rope, after: &Rope, record: &Transaction) -> Option<Self> {
        if self.current != record.before || self.bytes != before.len_bytes() {
            return None;
        }
        let mutation = Mutation::inspect(before, after, record)?;
        self.advance(&mutation, record.after)
    }

    fn advance(mut self, mutation: &Mutation, next: Stamp) -> Option<Self> {
        if next.buffer != self.current.buffer
            || self.current.revision.checked_add(1)? != next.revision
        {
            return None;
        }
        let base_start = self.base_byte(mutation.row, mutation.start)?;
        let slot = self.slot(mutation.row)?;
        let previous = self.rows[slot];
        if previous.is_some_and(|row| row.base_start != base_start) {
            return None;
        }
        self.rows[slot] = Some(Row {
            index: mutation.row,
            base_start,
            delta: previous
                .map_or(0, |row| row.delta)
                .checked_add(mutation.delta)?,
        });
        self.bytes = self.bytes.checked_add_signed(mutation.delta)?;
        self.check_deltas()?;
        self.current = next;
        Some(self)
    }

    fn slot(&self, row: usize) -> Option<usize> {
        self.rows
            .iter()
            .position(|slot| slot.is_some_and(|item| item.index == row))
            .or_else(|| self.rows.iter().position(Option::is_none))
    }

    /// Line starts map through strictly preceding rows. Bytes within a dirty
    /// row are deliberately not claimed by this mapping.
    fn base_byte(&self, row: usize, current: usize) -> Option<usize> {
        current.checked_add_signed(self.delta_before(row)?.checked_neg()?)
    }

    fn check_deltas(&self) -> Option<()> {
        // Check every prefix independently of the insertion order of row slots.
        for row in self.rows.iter().flatten() {
            let current = self.current_byte(row.index, row.base_start)?;
            self.base_byte(row.index, current)?;
        }
        (self.base_byte(usize::MAX, self.bytes)? == self.base_bytes).then_some(())
    }

    fn current_byte(&self, row: usize, base: usize) -> Option<usize> {
        base.checked_add_signed(self.delta_before(row)?)
    }

    fn delta_before(&self, row: usize) -> Option<isize> {
        self.rows
            .iter()
            .flatten()
            .filter(|item| item.index < row)
            .try_fold(0isize, |sum, item| sum.checked_add(item.delta))
    }
}

impl Buffer {
    pub(super) fn prepare_presentation(
        &self,
        text: &Rope,
        record: &Transaction,
    ) -> Option<Mapping> {
        let base = self.presentation_base()?;
        let mapping = if base == self.stamp() {
            Mapping::exact(base, self.len_bytes())
        } else {
            self.presentation.filter(|mapping| mapping.base == base)?
        };
        mapping.prepare(&self.text, text, record)
    }

    fn presentation_base(&self) -> Option<Stamp> {
        if self.is_org() {
            let outline = self.org_outline.as_ref()?;
            outline.indentation_at(0)?;
            return Some(outline.stamp);
        }
        self.markdown_observation().map(|value| value.stamp)
    }
}
