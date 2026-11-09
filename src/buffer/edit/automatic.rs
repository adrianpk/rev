//! Explicit native history boundaries for one automatic editing iteration.
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{Buffer, Error, History, Transaction};

pub(crate) const GENERATED_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct AutomaticGroup {
    budget: Arc<AtomicUsize>,
}

impl AutomaticGroup {
    pub fn new(budget: Arc<AtomicUsize>) -> Arc<Self> {
        Arc::new(Self { budget })
    }

    pub(crate) fn charge(&self, bytes: usize) -> Result<(), Error> {
        self.budget
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
                used.checked_add(bytes)
                    .filter(|total| *total <= GENERATED_BYTES)
            })
            .map(|_| ())
            .map_err(|_| {
                self.budget.store(usize::MAX, Ordering::Relaxed);
                Error::Capacity
            })
    }
}

impl crate::buffer::Buffers {
    pub(crate) fn automatic_group(&mut self, group: Option<Arc<AutomaticGroup>>) {
        let mut buffers = self.entries.values_mut();
        let Some(last) = buffers.next_back() else {
            return;
        };
        for buffer in buffers {
            buffer.break_typing();
            buffer.automatic = group.clone();
        }
        last.break_typing();
        last.automatic = group;
    }
}

impl Transaction {
    pub(super) fn same_automatic(&self, other: &Self) -> bool {
        match (&self.automatic, &other.automatic) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }
}

impl Buffer {
    /// Count native primitive records in the next logical undo/redo unit.
    /// Primitive replay preserves each checked mapping, mark and source anchor.
    pub(crate) fn history_unit(&self, direction: History) -> usize {
        let stack = match direction {
            History::Undo => &self.history,
            History::Redo => &self.redo,
        };
        let Some(last) = stack.back() else {
            return 0;
        };
        if last.automatic.is_none() {
            return 1;
        }
        stack
            .iter()
            .rev()
            .take_while(|record| last.same_automatic(record))
            .count()
    }
}
