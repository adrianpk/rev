use std::path::Path;

use crate::config::{Grammar, Snapshot};
use crate::syntax::Syntax;

use super::Buffer;

impl Buffer {
    /// Resolve the first visiting basename from a supplied immutable snapshot.
    /// This query performs no file IO, syntax work or text/history mutation.
    pub fn language<'a>(&'a self, settings: &'a Snapshot) -> &'a str {
        if let Some(language) = &self.source_language {
            return language;
        }
        self.file
            .as_ref()
            .map(|file| settings.language(Path::new(file.basename())))
            .unwrap_or("plain")
    }

    pub fn syntax(&self) -> &Syntax {
        &self.syntax
    }

    pub fn mode(&self) -> &'static str {
        if self.source_language.is_some() {
            if self.syntax.mode(self.stamp()) == "Rust" {
                "Org Src Rust"
            } else {
                "Org Src"
            }
        } else if self.is_transient() {
            "Capture"
        } else if self.is_org() {
            "Org"
        } else {
            self.syntax.mode(self.stamp())
        }
    }

    /// Refresh derived syntax outside atomic text publication. Failure changes
    /// only presentation; all accepted text, marks, views and history remain.
    pub fn refresh_syntax(&mut self) {
        let stamp = self.stamp();
        if self.prepare_syntax() {
            self.syntax.update(self.text.slice(..), stamp);
        }
    }

    /// Apply a finite progress policy to the same parser boundary as normal
    /// refresh. A failed attempt remains Text until another accepted revision.
    pub fn parse_with(&mut self, options: tree_sitter::ParseOptions<'_>) {
        let stamp = self.stamp();
        if self.prepare_syntax() {
            self.syntax.update_with(self.text.slice(..), stamp, options);
        }
    }

    // Parser admission is derived metadata. Dropping it above the byte cutoff
    // removes stale styles; shrinking below the cutoff starts from current text.
    // Parse failure keeps active=true, retaining once-per-revision retry bounds.
    fn prepare_syntax(&mut self) -> bool {
        let admitted = self.syntax_policy.enabled
            && self.mode_choice.is_none()
            && !self.is_org()
            && self.grammar == Grammar::Rust
            && (self.syntax_policy.max_bytes == 0
                || self.len_bytes() <= self.syntax_policy.max_bytes);
        if admitted != self.syntax_active {
            self.syntax = if admitted {
                Syntax::rust(Some(tree_sitter_rust::HIGHLIGHTS_QUERY))
            } else {
                Syntax::default()
            };
            self.syntax_active = admitted;
        }
        admitted
    }
}
