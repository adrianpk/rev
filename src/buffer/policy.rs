use crate::config::{Editor, Snapshot};

use super::Buffer;

impl Buffer {
    /// The effective editing/display policy belongs to this buffer, shared by
    /// every view. Widths retain the snapshot's admitted 1..=32 bounds.
    pub fn policy(&self) -> &Editor {
        &self.policy
    }

    /// Resolve from a supplied snapshot without reloading files or changing
    /// canonical text, revisions, marks, saved states, syntax or undo history.
    /// Views discard cached display goals before their next reveal/navigation.
    pub fn apply_settings(&mut self, settings: &Snapshot) {
        self.policy = settings.editor(self.language(settings));
    }
}
