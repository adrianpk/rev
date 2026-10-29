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
    /// History retention takes effect on the next completed edit; existing
    /// undo/redo traversal remains intact until that publication.
    /// Views discard cached display goals before their next reveal/navigation.
    pub fn apply_settings(&mut self, settings: &Snapshot) {
        let language = self.language(settings);
        let policy = settings.editor(language);
        let grammar = settings.grammar(language);
        self.policy = policy;
        self.history_policy = settings.settings().history.clone();
        if self.syntax_policy != settings.settings().syntax {
            self.org_attempt = None;
        }
        self.syntax_policy = settings.settings().syntax.clone();
        self.grammar = grammar;
        self.apply_mode(settings);
        self.refresh_syntax();
    }
}
