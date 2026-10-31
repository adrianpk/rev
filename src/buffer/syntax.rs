use std::path::Path;

use crate::config::{Grammar, Snapshot};
use crate::syntax::Syntax;

use super::Buffer;

impl Buffer {
    pub(super) fn selected_grammar(&self, grammar: Grammar) -> Grammar {
        if grammar == Grammar::Typescript
            && self.file.as_ref().is_some_and(|file| {
                Path::new(file.basename())
                    .extension()
                    .is_some_and(|ext| ext == "tsx")
            })
        {
            Grammar::Tsx
        } else {
            grammar
        }
    }
    pub(crate) fn syntax_language(&self) -> &str {
        &self.language_identity
    }
    pub(crate) fn completion_grammar(&self) -> Grammar {
        self.grammar
    }
    pub(super) fn reset_syntax(&mut self) {
        self.syntax_generation = self
            .syntax_generation
            .checked_add(1)
            .expect("syntax generation exhausted");
        self.syntax = Syntax::default();
        self.syntax_active = false;
    }
    pub(crate) fn syntax_generation(&self) -> u64 {
        self.syntax_generation
    }
    pub(crate) fn completion_available(&self) -> bool {
        self.completion_supported() && self.len_bytes() <= crate::completion::MAX_BYTES
    }
    pub(crate) fn completion_supported(&self) -> bool {
        if self.is_org() {
            return self.org_colors_admitted() && !self.is_readonly();
        }
        self.syntax_active
            && (self.grammar != Grammar::Plain || self.external_applied)
            && self.mode_choice.is_none()
            && !self.is_org()
            && !self.is_readonly()
    }
    pub(crate) fn completion_snapshot(&mut self) -> Result<(ropey::Rope, Syntax), String> {
        if self.is_readonly() {
            return Err("buffer is read-only".into());
        }
        if self.len_bytes() > crate::completion::MAX_BYTES {
            return Err("Native completion buffer exceeds 2 MiB".into());
        }
        if self.is_org() && self.org_colors_admitted() {
            return Ok((self.text.clone(), Syntax::default()));
        }
        if !self.prepare_syntax() {
            return Err("Native completion syntax is unavailable".into());
        }
        let syntax = self
            .syntax
            .take_work()
            .ok_or("Native syntax work is already owned")?;
        Ok((self.text.clone(), syntax))
    }

    pub(crate) fn restore_completion_syntax(&mut self, syntax: Syntax) {
        if self.syntax.working() {
            self.syntax = syntax;
        }
    }

    pub(crate) fn abandon_completion_syntax(&mut self) {
        if self.syntax.working() {
            self.syntax = Syntax::default();
            self.syntax_active = false;
        }
    }

    #[cfg(test)]
    pub(crate) fn completion_symbols(
        &mut self,
        deadline: std::time::Instant,
    ) -> Result<Vec<crate::completion::Symbol>, String> {
        if self.is_readonly() {
            return Err("buffer is read-only".into());
        }
        if self.len_bytes() > crate::completion::MAX_BYTES {
            return Err("Native completion buffer exceeds 2 MiB".into());
        }
        let mut progress = |_: &tree_sitter::ParseState| {
            if std::time::Instant::now() >= deadline {
                std::ops::ControlFlow::Break(())
            } else {
                std::ops::ControlFlow::Continue(())
            }
        };
        self.parse_with(tree_sitter::ParseOptions::new().progress_callback(&mut progress));
        self.syntax
            .declarations(self.text.slice(..), self.stamp(), deadline)
    }

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
            self.syntax.source_mode(self.stamp())
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
            && (self.grammar != Grammar::Plain || self.external.is_some())
            && (self.syntax_policy.max_bytes == 0
                || self.len_bytes() <= self.syntax_policy.max_bytes);
        let assets = self.external_assets(admitted);
        let external = assets.is_some();
        if admitted != self.syntax_active || external != self.external_applied {
            self.syntax_generation = self
                .syntax_generation
                .checked_add(1)
                .expect("syntax generation exhausted");
            self.syntax = if admitted {
                assets.as_ref().map_or_else(
                    || Syntax::bundled(self.grammar),
                    |assets| Syntax::external(assets),
                )
            } else {
                Syntax::default()
            };
            self.syntax_active = admitted;
            self.external_applied = external;
        }
        admitted
    }
}

impl Buffer {
    fn external_assets(
        &self,
        admitted: bool,
    ) -> Option<std::sync::Arc<crate::syntax::assets::Assets>> {
        let handle = self.external.as_ref().filter(|_| admitted)?;
        handle.start();
        handle.admitted()
    }

    /// None means unregistered or not yet admitted; a completed failure is
    /// retained for this session while bundled/plain editing stays available.
    pub fn external_grammar_status(&self) -> Option<Result<(), &str>> {
        self.external.as_ref()?.status()
    }

    fn assets_pending(&self) -> bool {
        self.external.as_ref().is_some_and(|handle| {
            handle.pending() || (!self.external_seen && handle.status().is_some())
        })
    }

    fn poll_assets(&mut self) -> bool {
        if self.external_seen || self.external_grammar_status().is_none() {
            return false;
        }
        self.external_seen = true;
        self.refresh_syntax();
        true
    }
}

impl super::Buffers {
    pub(crate) fn assets_pending(&self) -> bool {
        self.entries.values().any(Buffer::assets_pending)
    }
    pub(crate) fn poll_assets(&mut self) -> bool {
        let mut changed = false;
        for buffer in self.entries.values_mut() {
            changed |= buffer.poll_assets();
        }
        changed
    }
}
