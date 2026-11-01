use crate::location::Anchor;
use std::collections::VecDeque;

pub(crate) const LIMIT: usize = 256;

/// View-owned, session-only traversal state. Splits copy its bounded handles
/// and index; each view then changes its own branch independently.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct History {
    entries: VecDeque<Anchor>,
    index: usize,
}

pub(crate) struct Record {
    pub origin: Anchor,
    pub destination: Anchor,
}

impl History {
    pub(crate) fn record(&mut self, record: Record) {
        self.entries.truncate(self.index.saturating_add(1));
        self.append(record.origin);
        self.append(record.destination);
        self.index = self.entries.len().saturating_sub(1);
    }

    fn append(&mut self, anchor: Anchor) {
        if self
            .entries
            .back()
            .is_some_and(|last| last.same_point(&anchor))
        {
            return;
        }
        self.entries.push_back(anchor);
        if self.entries.len() > LIMIT {
            self.entries.pop_front();
        }
    }

    pub(crate) fn traversal(
        &self,
        backward: bool,
        count: usize,
        current: Anchor,
    ) -> Option<(Self, Anchor)> {
        let mut candidate = self.clone();
        if backward
            && candidate.index + 1 == candidate.entries.len()
            && let Some(location) = candidate.entries.get_mut(candidate.index)
        {
            *location = current;
        }
        let (index, target) = candidate.target(backward, count)?;
        candidate.accept(index);
        Some((candidate, target))
    }

    fn target(&self, backward: bool, count: usize) -> Option<(usize, Anchor)> {
        let index = if backward {
            self.index.saturating_sub(count)
        } else {
            self.index
                .saturating_add(count)
                .min(self.entries.len().checked_sub(1)?)
        };
        (index != self.index).then(|| (index, self.entries[index].clone()))
    }

    pub(crate) fn accept(&mut self, index: usize) {
        self.index = index;
    }
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        buffer::{Buffers, edit::Point},
        coordinates::Scalar,
    };

    fn anchor(buffer: &crate::buffer::Buffer, at: usize) -> Anchor {
        buffer
            .locations
            .lock()
            .unwrap()
            .capture(
                buffer,
                Point {
                    stamp: buffer.stamp(),
                    position: Scalar(at),
                },
            )
            .unwrap()
    }

    // Retention is exactly bounded, copied views branch independently, and
    // evicted/deleted histories release their native location handles.
    #[test]
    fn bounded_history() {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(&"x".repeat(LIMIT + 64)).unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut history = History::default();
        for at in 0..LIMIT + 32 {
            history.record(Record {
                origin: anchor(buffer, at),
                destination: anchor(buffer, at + 1),
            });
        }
        assert_eq!(history.len(), LIMIT);
        let (mut copy, target) = history
            .traversal(true, usize::MAX, anchor(buffer, LIMIT + 32))
            .unwrap();
        assert_eq!(target.source().point.position, Scalar(33));
        drop(target);
        copy.record(Record {
            origin: anchor(buffer, 33),
            destination: anchor(buffer, 0),
        });
        assert_eq!(copy.len(), 2);
        assert_eq!(history.len(), LIMIT);
        drop(copy);
        drop(history);
        assert!(
            buffer
                .locations
                .lock()
                .unwrap()
                .anchors(buffer.stamp())
                .unwrap()
                .is_empty()
        );
    }
}
