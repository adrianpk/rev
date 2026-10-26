use std::collections::VecDeque;

use ropey::Rope;

use crate::buffer::{Buffer, Span, Stamp, edit::Error};
use crate::coordinates::Scalar;
use crate::window::ViewId;

#[derive(Clone, Copy)]
struct Continuation {
    stamp: Stamp,
    view: ViewId,
    point: Scalar,
}

/// Owned linear text, newest first, with a fixed entry bound.
pub(crate) struct KillRing {
    entries: VecDeque<Rope>,
    continuation: Option<Continuation>,
}

/// Fully composed before editing; publication requires no further allocation.
pub(crate) struct Entry {
    text: Rope,
    coalesced: bool,
}

impl Default for KillRing {
    fn default() -> Self {
        Self {
            entries: VecDeque::with_capacity(32),
            continuation: None,
        }
    }
}

impl KillRing {
    pub(crate) fn text(&self) -> Option<&Rope> {
        self.entries.front()
    }

    pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = &Rope> + ExactSizeIterator {
        self.entries.iter()
    }

    pub(crate) fn copy(&mut self, text: Rope) {
        self.break_chain();
        self.store(Entry {
            text,
            coalesced: false,
        });
    }

    pub(crate) fn break_chain(&mut self) {
        self.continuation = None;
    }

    /// Prepare owned fragments before text publication; a rejected edit drops
    /// this candidate without replacing the previously retained kill text.
    pub(crate) fn prepare(
        &self,
        buffer: &Buffer,
        view: ViewId,
        source: &Span,
        backward: bool,
    ) -> Result<Entry, Error> {
        let mut fragment: Rope = buffer.slice(source)?.into();
        let point = Scalar(if backward {
            source.range.end
        } else {
            source.range.start
        });
        let Some(previous) = self
            .text()
            .filter(|_| self.continues(source.stamp, view, point))
        else {
            return Ok(Entry {
                text: fragment,
                coalesced: false,
            });
        };
        let text = if backward {
            fragment.append(previous.clone());
            fragment
        } else {
            let mut text = previous.clone();
            text.append(fragment);
            text
        };
        Ok(Entry {
            text,
            coalesced: true,
        })
    }

    fn continues(&self, stamp: Stamp, view: ViewId, point: Scalar) -> bool {
        self.continuation.is_some_and(|previous| {
            previous.stamp == stamp && previous.view == view && previous.point == point
        })
    }

    pub(crate) fn publish(&mut self, entry: Entry, stamp: Stamp, view: ViewId, point: Scalar) {
        self.store(entry);
        self.continuation = Some(Continuation { stamp, view, point });
    }

    fn store(&mut self, entry: Entry) {
        if entry.coalesced {
            self.entries.pop_front();
        } else if self.entries.len() == 32 {
            self.entries.pop_back();
        }
        self.entries.push_front(entry.text);
    }
}
