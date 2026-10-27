use std::path::Path;

use crate::config::{Grammar, Snapshot};
use crate::syntax::Syntax;

use super::Buffer;

impl Buffer {
    /// Resolve the first visiting basename from a supplied immutable snapshot.
    /// This query performs no file IO, syntax work or text/history mutation.
    pub fn language<'a>(&self, settings: &'a Snapshot) -> &'a str {
        self.file
            .as_ref()
            .map(|file| settings.language(Path::new(file.basename())))
            .unwrap_or("plain")
    }

    pub fn syntax(&self) -> &Syntax {
        &self.syntax
    }

    pub fn mode(&self) -> &'static str {
        self.syntax.mode(self.stamp())
    }

    /// Refresh derived syntax outside atomic text publication. Failure changes
    /// only presentation; all accepted text, marks, views and history remain.
    pub fn refresh_syntax(&mut self) {
        let stamp = self.stamp();
        self.syntax.update(self.text.slice(..), stamp);
    }

    /// Apply a finite progress policy to the same parser boundary as normal
    /// refresh. A failed attempt remains Text until another accepted revision.
    pub fn parse_with(&mut self, options: tree_sitter::ParseOptions<'_>) {
        let stamp = self.stamp();
        self.syntax.update_with(self.text.slice(..), stamp, options);
    }

    pub(super) fn select_syntax(&mut self) {
        let settings = Snapshot::default();
        let rust = settings.grammar(self.language(&settings)) == Grammar::Rust;
        self.syntax = if rust {
            Syntax::rust(Some(tree_sitter_rust::HIGHLIGHTS_QUERY))
        } else {
            Syntax::default()
        };
        self.refresh_syntax();
    }
}
