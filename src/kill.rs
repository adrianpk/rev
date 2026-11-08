use std::collections::VecDeque;
use std::sync::Arc;
pub(crate) const EXPANSION_BYTES: usize = 64 * 1024 * 1024;

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

/// Owned text transfers, newest first, with a fixed entry bound.
pub(crate) struct KillRing {
    entries: VecDeque<Text>,
    policy: crate::config::KillRing,
    continuation: Option<Continuation>,
    generation: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shape {
    Linear,
    Line,
    Block,
}

pub(crate) struct Transfer {
    pub view: ViewId,
    pub backward: bool,
    pub shape: Shape,
}

struct Text {
    text: Rope,
    shape: Shape,
    block: Option<Arc<crate::rectangle::Rectangle>>,
}

/// Fully composed before editing; publication requires no further allocation.
pub(crate) struct Entry {
    text: Rope,
    coalesced: bool,
    shape: Shape,
    block: Option<Arc<crate::rectangle::Rectangle>>,
}

impl KillRing {
    pub(crate) fn with_settings(settings: &crate::config::Snapshot) -> Self {
        let policy = settings.settings().kill_ring.clone();
        Self {
            entries: VecDeque::with_capacity(policy.capacity),
            policy,
            continuation: None,
            generation: Some(0),
        }
    }

    pub(crate) fn generation(&self) -> Option<u64> {
        self.generation
    }

    pub(crate) fn text(&self) -> Option<&Rope> {
        self.entries.front().map(|entry| &entry.text)
    }

    pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = &Rope> + ExactSizeIterator {
        self.entries.iter().map(|entry| &entry.text)
    }

    pub(crate) fn older(&self, index: usize) -> Option<(usize, &Rope)> {
        self.rotate(index, 1)
    }

    pub(crate) fn entry(&self, index: usize) -> Option<&Rope> {
        self.entries.get(index).map(|entry| &entry.text)
    }

    pub(crate) fn rotate(&self, index: usize, distance: i32) -> Option<(usize, &Rope)> {
        let len = i64::try_from(self.entries.len()).ok()?;
        if len == 0 {
            return None;
        }
        let next = (i64::try_from(index).ok()? + i64::from(distance)).rem_euclid(len) as usize;
        Some((next, &self.entries.get(next)?.text))
    }

    pub(crate) fn copy(&mut self, text: Rope) {
        self.break_chain();
        self.store(Entry {
            text,
            coalesced: false,
            shape: Shape::Linear,
            block: None,
        });
    }

    pub(crate) fn shape(&self) -> Option<Shape> {
        self.entries.front().map(|entry| entry.shape)
    }

    pub(crate) fn prepare_transfer(
        &self,
        buffer: &Buffer,
        source: &Span,
        transfer: Transfer,
    ) -> Result<Entry, Error> {
        self.prepare_shaped(buffer, source, transfer)
    }

    pub(crate) fn block(&self) -> Option<Arc<crate::rectangle::Rectangle>> {
        self.entries.front()?.block.clone()
    }

    pub(crate) fn prepare_block(&self, block: Arc<crate::rectangle::Rectangle>) -> Entry {
        Entry {
            text: Rope::from_str(&block.rows().join("\n")),
            coalesced: false,
            shape: Shape::Block,
            block: Some(block),
        }
    }

    pub(crate) fn publish_transfer(&mut self, entry: Entry) {
        self.break_chain();
        self.store(entry);
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
        self.prepare_shaped(
            buffer,
            source,
            Transfer {
                view,
                backward,
                shape: Shape::Linear,
            },
        )
    }

    fn prepare_shaped(
        &self,
        buffer: &Buffer,
        source: &Span,
        transfer: Transfer,
    ) -> Result<Entry, Error> {
        let Transfer {
            view,
            backward,
            shape,
        } = transfer;
        let mut fragment: Rope = buffer.slice(source)?.into();
        let point = Scalar(if backward {
            source.range.end
        } else {
            source.range.start
        });
        let Some(previous) = self
            .text()
            .filter(|_| self.continues(source.stamp, view, point, shape))
        else {
            return Ok(Entry {
                text: fragment,
                coalesced: false,
                shape,
                block: None,
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
            shape,
            block: None,
        })
    }

    fn continues(&self, stamp: Stamp, view: ViewId, point: Scalar, shape: Shape) -> bool {
        self.policy.coalesce
            && self.shape() == Some(shape)
            && self.continuation.is_some_and(|previous| {
                previous.stamp == stamp && previous.view == view && previous.point == point
            })
    }

    pub(crate) fn publish(&mut self, entry: Entry, stamp: Stamp, view: ViewId, point: Scalar) {
        self.store(entry);
        self.continuation = Some(Continuation { stamp, view, point });
    }

    fn store(&mut self, entry: Entry) {
        self.generation = self.generation.and_then(|value| value.checked_add(1));
        if entry.coalesced {
            self.entries.pop_front();
        } else if self.entries.len() == self.policy.capacity {
            self.entries.pop_back();
        }
        self.entries.push_front(Text {
            text: entry.text,
            shape: entry.shape,
            block: entry.block,
        });
    }
}
