//! Explicit, session-owned native assets. IO, loading and query compilation run
//! on one lazy worker; published outcomes are immutable and are never retried.
mod admission;
mod context;
mod queue;

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock, atomic::AtomicBool};

use crate::config::{External, Language};

pub(crate) struct Assets {
    pub language: tree_sitter::Language,
    pub highlights: Arc<tree_sitter::Query>,
    pub completions: Arc<tree_sitter::Query>,
}

struct Entry {
    registration: External,
    root: Option<std::path::PathBuf>,
    started: AtomicBool,
    outcome: OnceLock<Result<Arc<Assets>, String>>,
}

#[derive(Clone, Default)]
pub(crate) struct Registry {
    entries: BTreeMap<String, Arc<Entry>>,
    queue: Arc<queue::Queue>,
}

#[derive(Clone)]
pub(crate) struct Handle {
    entry: Arc<Entry>,
    queue: Arc<queue::Queue>,
}

impl Registry {
    pub fn new(languages: &BTreeMap<String, Language>, root: Option<&std::path::Path>) -> Self {
        let mut registry = Self::default();
        for (id, language) in languages {
            if let Some(registration) = &language.external {
                registry.entries.insert(
                    id.clone(),
                    Arc::new(Entry {
                        registration: registration.clone(),
                        root: root.map(std::path::Path::to_path_buf),
                        started: AtomicBool::new(false),
                        outcome: OnceLock::new(),
                    }),
                );
            }
        }
        registry
    }

    pub fn get(&self, language: &str) -> Option<Handle> {
        Some(Handle {
            entry: std::sync::Arc::clone(self.entries.get(language)?),
            queue: std::sync::Arc::clone(&self.queue),
        })
    }
}

impl std::fmt::Debug for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.entries
                    .iter()
                    .map(|(id, entry)| (id, &entry.registration)),
            )
            .finish()
    }
}
impl PartialEq for Registry {
    fn eq(&self, other: &Self) -> bool {
        self.entries.len() == other.entries.len()
            && self.entries.iter().all(|(id, entry)| {
                other.entries.get(id).is_some_and(|other| {
                    entry.registration == other.registration && entry.root == other.root
                })
            })
    }
}

impl Handle {
    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.entry, &other.entry)
    }
    pub fn start(&self) {
        if !self
            .entry
            .started
            .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            self.queue.schedule(std::sync::Arc::clone(&self.entry));
        }
    }
    pub fn pending(&self) -> bool {
        self.entry
            .started
            .load(std::sync::atomic::Ordering::Acquire)
            && self.entry.outcome.get().is_none()
    }
    pub fn status(&self) -> Option<Result<(), &str>> {
        self.entry
            .outcome
            .get()
            .map(|outcome| outcome.as_ref().map(|_| ()).map_err(String::as_str))
    }
    pub(crate) fn admitted(&self) -> Option<Arc<Assets>> {
        self.entry.outcome.get()?.as_ref().ok().cloned()
    }
}

#[cfg(test)]
mod tests;
