use crate::config::{Editor, Snapshot};

use super::Buffer;

impl Buffer {
    /// Presentation is shared by this buffer's views and never enters history.
    pub fn soft_wrap(&self) -> bool {
        self.wrap_override.unwrap_or(self.policy.soft_wrap)
    }

    /// Toggle this live buffer only; new buffers retain the startup default.
    pub fn toggle_soft_wrap(&mut self) -> bool {
        let enabled = !self.soft_wrap();
        self.wrap_override = Some(enabled);
        enabled
    }

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
        self.org_indent = settings.settings().org.indent;
        let language = self.language(settings).to_owned();
        let policy = settings.editor(&language);
        let grammar = self.selected_grammar(settings.grammar(&language));
        self.policy = policy;
        self.history_policy = settings.settings().history.clone();
        if self.text_byte_limit.is_some() {
            self.history_policy.group_limit = self.history_policy.group_limit.min(16);
            self.history_policy.payload_target_bytes =
                self.history_policy.payload_target_bytes.min(131072);
        }
        if self.syntax_policy != settings.settings().syntax {
            self.org_attempt = None;
            self.reset_markdown();
        }
        self.syntax_policy = settings.settings().syntax.clone();
        let external = settings.external(&language);
        let same = match (&self.external, &external) {
            (Some(old), Some(new)) => old.same(new),
            (None, None) => true,
            _ => false,
        };
        if !same {
            self.external = external;
            self.external_seen = false;
            self.external_applied = false;
        }
        if !same || self.grammar != grammar || self.language_identity != language {
            self.reset_syntax();
        }
        self.grammar = grammar;
        self.language_identity = language;
        self.apply_mode(settings);
        self.refresh_syntax();
    }
}
