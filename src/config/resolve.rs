use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::{Diagnostic, Document, Editor, Grammar, Settings, Sources, project, validate};

/// One validated, immutable configuration value. Only shared access to its
/// settings is exposed; language policies derive from this same frozen value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    settings: Settings,
    sources: Vec<PathBuf>,
    origins: BTreeMap<Vec<String>, usize>,
}

impl Snapshot {
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Overlay the selected language after all global layers, without file IO,
    /// defaulting omitted siblings or requiring a grammar for that identity.
    pub fn editor(&self, language: &str) -> Editor {
        project::language(&self.settings.editor, self.settings.language.get(language))
    }

    /// Rust identity selects the bundled grammar by default; all others are plain.
    pub fn grammar(&self, language: &str) -> Grammar {
        self.settings
            .language
            .get(language)
            .and_then(|language| language.grammar)
            .unwrap_or_else(|| {
                if language == "rust" {
                    Grammar::Rust
                } else {
                    Grammar::Plain
                }
            })
    }

    /// The last explicitly assigning source, or None for a compiled default or
    /// absent language leaf. Dynamic keys remain separate path components.
    pub fn origin(&self, key: &[&str]) -> Option<&Path> {
        let key: Vec<String> = key.iter().map(|part| (*part).to_owned()).collect();
        self.origins.get(&key).map(|id| self.sources[*id].as_path())
    }
}

/// Resolve the loader's postorder against compiled defaults. Every source and
/// the merged candidate validate before any snapshot is returned. Includes are
/// traversal metadata, so their arrays never become another discovery pass.
pub fn resolve(sources: &Sources) -> Result<Snapshot, Diagnostic> {
    order(sources)?;
    let mut document = Document::default();
    let mut origins = BTreeMap::new();
    for id in &sources.order {
        let source = &sources.sources[*id];
        validate::document(&source.document)
            .map_err(|failure| failure_at(&source.path, failure))?;
        for key in document.overlay(&source.document) {
            origins.insert(key, *id);
        }
    }
    validate::document(&document).map_err(|failure| {
        let path = origins
            .get(&failure.key)
            .map(|id| sources.sources[*id].path.as_path())
            .unwrap_or(Path::new("config.toml"));
        failure_at(path, failure)
    })?;
    Ok(Snapshot {
        settings: project::settings(&document),
        sources: sources
            .sources
            .iter()
            .map(|source| source.path.clone())
            .collect(),
        origins,
    })
}

fn failure_at(path: &Path, failure: validate::Failure) -> Diagnostic {
    let mut diagnostic = Diagnostic::new(path, failure.message);
    diagnostic.key = failure.key;
    diagnostic
}

fn order(sources: &Sources) -> Result<(), Diagnostic> {
    let path = sources
        .sources
        .first()
        .map(|source| source.path.as_path())
        .unwrap_or(Path::new("config.toml"));
    if sources.sources.len() > super::load::MAX_SOURCES
        || sources.order.len() != sources.sources.len()
    {
        return Err(Diagnostic::new(
            path,
            "invalid source occurrence order".into(),
        ));
    }
    let mut seen = BTreeSet::new();
    for id in &sources.order {
        if *id >= sources.sources.len() || !seen.insert(*id) {
            return Err(Diagnostic::new(
                path,
                "invalid source occurrence order".into(),
            ));
        }
    }
    Ok(())
}
