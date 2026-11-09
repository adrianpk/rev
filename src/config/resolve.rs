use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::{Diagnostic, Document, Editor, Grammar, Settings, Sources, project, validate};

/// One validated, immutable configuration value. Only shared access to its
/// settings is exposed; language policies derive from this same frozen value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    settings: Settings,
    plugins: std::sync::Arc<crate::plugins::declarations::Registry>,
    assets: crate::syntax::assets::Registry,
    keymaps: crate::input::keymap::Effective,
    sources: Vec<PathBuf>,
    origins: BTreeMap<Vec<String>, usize>,
    root: Option<PathBuf>,
}

impl Snapshot {
    /// Activate one explicitly published definition; other settings and startup
    /// keymaps remain frozen, including other definitions changed on disk.
    pub(crate) fn ai_definition(&mut self, name: &str, incoming: &Self) {
        let definition = incoming
            .settings
            .ai
            .commands
            .get(name)
            .expect("published AI definition");
        self.settings
            .ai
            .commands
            .insert(name.to_owned(), definition.clone());
        let key = vec!["ai".into(), "commands".into(), name.to_owned()];
        if let Some(source) = incoming.origins.get(&key) {
            let path = &incoming.sources[*source];
            let id = self
                .sources
                .iter()
                .position(|p| p == path)
                .unwrap_or_else(|| {
                    self.sources.push(path.clone());
                    self.sources.len() - 1
                });
            self.origins.insert(key, id);
        }
    }

    /// Restrict live application to LSP and association leaves. Other settings,
    /// grammar registrations and the startup interaction profile remain frozen.
    pub(crate) fn lsp_delta(&mut self, incoming: &Self) {
        self.settings.lsp = incoming.settings.lsp.clone();
        self.settings.files.associations = incoming.settings.files.associations.clone();
        self.origins.retain(|key, _| !lsp_key(key));
        for (key, source) in incoming.origins.iter().filter(|(key, _)| lsp_key(key)) {
            let path = &incoming.sources[*source];
            let id = self
                .sources
                .iter()
                .position(|p| p == path)
                .unwrap_or_else(|| {
                    self.sources.push(path.clone());
                    self.sources.len() - 1
                });
            self.origins.insert(key.clone(), id);
        }
        // Drop paths used only by a previous LSP graph. Repeated reloads cannot
        // accumulate historical include names in a long-running session.
        let previous = std::mem::take(&mut self.sources);
        for id in self.origins.values_mut() {
            let path = &previous[*id];
            *id = self
                .sources
                .iter()
                .position(|p| p == path)
                .unwrap_or_else(|| {
                    self.sources.push(path.clone());
                    self.sources.len() - 1
                });
        }
    }
    /// The selected user root, including explicit spellings and absent roots.
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// Retain the selected user root without admitting it as a configuration
    /// source. Persistence must still capture and validate its current graph.
    pub fn with_root(mut self, root: &Path) -> Self {
        self.root = Some(root.to_owned());
        self
    }
    /// Resolve explicit grammar paths under a supplied, already resolved rev
    /// data directory. Call before buffer construction; this performs no IO.
    pub fn with_data_root(mut self, data: &Path) -> Self {
        self.assets = crate::syntax::assets::Registry::new(
            &self.settings.language,
            Some(data.join("grammars")),
        );
        self
    }

    pub(crate) fn external(&self, language: &str) -> Option<crate::syntax::assets::Handle> {
        self.assets.get(language)
    }

    /// Complete admitted maps; user overrides are not activated by this accessor.
    pub fn keymaps(&self) -> &crate::input::keymap::Effective {
        &self.keymaps
    }

    /// Atomically admitted passive plugin metadata; access performs no I/O.
    pub fn plugins(&self) -> &crate::plugins::declarations::Registry {
        &self.plugins
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Overlay the selected language after all global layers, without file IO,
    /// defaulting omitted siblings or requiring a grammar for that identity.
    pub fn editor(&self, language: &str) -> Editor {
        project::language(&self.settings.editor, self.settings.language.get(language))
    }

    /// Bundled identities select their grammar by default; other identities are plain.
    pub fn grammar(&self, language: &str) -> Grammar {
        self.settings
            .language
            .get(language)
            .and_then(|language| language.grammar)
            .unwrap_or(match language {
                "rust" => Grammar::Rust,
                "c" => Grammar::C,
                "go" => Grammar::Go,
                "html" => Grammar::Html,
                "css" => Grammar::Css,
                "javascript" => Grammar::Javascript,
                "typescript" => Grammar::Typescript,
                "tsx" => Grammar::Tsx,
                _ => format_grammar(language),
            })
    }

    /// The last explicitly assigning source, or None for a compiled default or
    /// absent language leaf. Dynamic keys remain separate path components.
    pub fn origin(&self, key: &[&str]) -> Option<&Path> {
        let key: Vec<String> = key.iter().map(|part| (*part).to_owned()).collect();
        self.origins.get(&key).map(|id| self.sources[*id].as_path())
    }
}

fn lsp_key(key: &[String]) -> bool {
    key.first().is_some_and(|part| part == "lsp")
        || (key.first().is_some_and(|part| part == "files")
            && key.get(1).is_some_and(|part| part == "associations"))
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
    origins.retain(|key, _| retained_origin(&document, key));
    let keymaps = validate::candidate(&document).map_err(|failure| {
        let path = origins
            .get(&failure.key)
            .map(|id| sources.sources[*id].path.as_path())
            .unwrap_or(Path::new("config.toml"));
        failure_at(path, failure)
    })?;
    let settings = project::settings(&document);
    let origin = origins
        .get(&vec!["plugins".into(), "manifests".into()])
        .map(|id| sources.sources[*id].path.as_path())
        .unwrap_or(Path::new("config.toml"));
    let plugins = crate::plugins::declarations::Registry::load(&settings.plugins, origin).map_err(
        |error| {
            let mut diagnostic = Diagnostic::new(&error.source, error.message);
            diagnostic.key = vec!["plugins".into(), "manifests".into()];
            diagnostic
        },
    )?;
    for binding in plugins
        .packages()
        .flat_map(|p| p.manifest.bindings.iter())
        .chain(settings.plugins.keybindings.iter())
    {
        let sequence = crate::input::keymap::Sequence::parse(binding.keys.as_str())
            .expect("validated declaration sequence");
        keymaps
            .admit_plugin(
                crate::input::keymap::contributions::context(binding.context),
                &sequence,
            )
            .map_err(|message| Diagnostic::new(origin, message))?;
    }
    let root = super::Roots::from_env()
        .ok()
        .map(|roots| roots.data.join("grammars"));
    let assets = crate::syntax::assets::Registry::new(&settings.language, root);
    Ok(Snapshot {
        settings,
        plugins: std::sync::Arc::new(plugins),
        assets,
        keymaps,
        sources: sources
            .sources
            .iter()
            .map(|source| source.path.clone())
            .collect(),
        origins,
        root: sources.sources.first().map(|source| source.path.clone()),
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

fn retained_origin(document: &Document, key: &[String]) -> bool {
    let [root, context, sequence] = key else {
        return true;
    };
    if root != "keys" {
        return true;
    }
    let context = crate::input::keymap::Context::ALL
        .into_iter()
        .find(|map| map.name() == context);
    context.is_none_or(|context| document.keys.map(context).contains_key(sequence))
}

fn format_grammar(language: &str) -> Grammar {
    match language {
        "json" => Grammar::Json,
        "toml" => Grammar::Toml,
        _ => Grammar::Plain,
    }
}
