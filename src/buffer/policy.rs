use crate::config::{Editor, Snapshot};

use super::Buffer;

impl Buffer {
    /// The effective editing/display policy belongs to this buffer, shared by
    /// every view. Widths retain the snapshot's admitted 1..=32 bounds.
    pub fn policy(&self) -> &Editor {
        &self.policy
    }

    /// Resolve from a supplied snapshot without reloading files or changing
    /// canonical text, revisions, marks, saved states or undo history.
    /// Syntax presentation follows its admitted grammar and global cutoff.
    /// Views discard cached display goals before their next reveal/navigation.
    pub fn apply_settings(&mut self, settings: &Snapshot) {
        let language = self.language(settings);
        self.policy = settings.editor(language);
        self.syntax_policy = settings.settings().syntax.clone();
        self.grammar = settings.grammar(language);
        self.refresh_syntax();
    }
}
