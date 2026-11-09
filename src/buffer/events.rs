//! Metadata-only post-publication observations, independent of LSP snapshots.
use super::{Buffer, Buffers, Stamp};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Opened,
    Changed,
    Closed,
}
pub(crate) trait Observer: Send + Sync {
    /// Enqueue only. Native source/history/view publication cannot reenter IO.
    fn published(&self, kind: Kind, stamp: Stamp);
}
impl Buffer {
    pub(crate) fn notify_event(&self, kind: Kind) {
        if !self.is_transient()
            && self.terminal().is_none()
            && let Some(observer) = &self.events
        {
            observer.published(kind, self.stamp());
        }
    }
}
impl Buffers {
    pub(crate) fn observe_events(&mut self, observer: Arc<dyn Observer>) {
        for buffer in self.entries.values_mut() {
            buffer.events = Some(observer.clone());
        }
        self.events = Some(observer);
    }
}
