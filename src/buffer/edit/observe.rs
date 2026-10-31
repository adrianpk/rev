use super::{Buffer, Change, Stamp, Transaction};
use ropey::Rope;
use std::sync::Arc;

/// Immutable evidence of this publication, before history grouping or replay
/// can change its representation. It does not authorize another text mutation.
#[derive(Clone)]
pub(crate) struct Published {
    pub before: Stamp,
    pub after: Stamp,
    pub original: Rope,
    pub text: Rope,
    pub changes: Vec<Change>,
}

/// One private post-commit consumer. Implementations must only enqueue bounded
/// observations without waiting for IO or changing the canonical transaction.
pub(crate) trait Observer: Send + Sync {
    fn active(&self) -> bool;
    fn published(&self, publication: Published);
}

impl Published {
    pub(super) fn capture(original: &Rope, text: &Rope, record: &Transaction) -> Self {
        Self {
            before: record.before,
            after: record.after,
            original: original.clone(),
            text: text.clone(),
            changes: record.changes.clone(),
        }
    }
}

impl Buffer {
    pub(crate) fn observe(&mut self, observer: Option<Arc<dyn Observer>>) {
        self.observer = observer;
    }

    pub(crate) fn text_snapshot(&self) -> Rope {
        self.text.clone()
    }
}
