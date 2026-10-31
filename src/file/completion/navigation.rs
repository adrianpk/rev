use std::path::{Path, PathBuf};

use super::{Candidate, Provider, Query};

mod accept;
mod complete;

/// Retained path draft, component observation and independent list selection.
/// Only draft directory transitions or explicit refresh may observe disk.
#[derive(Debug)]
pub struct Navigation {
    base: PathBuf,
    home: Option<PathBuf>,
    draft: String,
    query: Option<Query>,
    provider: Option<Provider>,
    error: Option<String>,
    selected: usize,
    chosen: bool,
    top: usize,
}

impl Navigation {
    pub fn new(base: PathBuf, home: Option<PathBuf>) -> Self {
        let mut navigation = Self {
            base,
            home,
            draft: String::new(),
            query: None,
            provider: None,
            error: None,
            selected: 0,
            chosen: false,
            top: 0,
        };
        navigation.update("", true);
        navigation
    }

    /// Text-only filtering reuses the same directory snapshot. Cursor motion,
    /// resize and refused edits leave this state unchanged.
    pub fn update(&mut self, draft: &str, refresh: bool) {
        if self.draft == draft && !refresh {
            return;
        }
        if self.draft != draft {
            self.selected = 0;
            self.chosen = false;
            self.top = 0;
        }
        self.draft = draft.to_owned();
        match Query::parse(&self.base, self.home.as_deref(), draft) {
            Ok(query) => self.observe(query, refresh),
            Err(error) => {
                self.query = None;
                self.provider = None;
                self.error = Some(error.to_string());
            }
        }
    }

    fn observe(&mut self, query: Query, refresh: bool) {
        let changed = self
            .query
            .as_ref()
            .is_none_or(|previous| previous.directory() != query.directory());
        if changed || refresh {
            match Provider::observe(query.directory()) {
                Ok(provider) => {
                    self.provider = Some(provider);
                    self.error = None;
                }
                Err(error) => {
                    self.provider = None;
                    self.error = Some(error.to_string());
                }
            }
        }
        self.query = Some(query);
        self.selected = self.selected.min(self.candidates().len().saturating_sub(1));
    }

    pub fn candidates(&self) -> Vec<&Candidate> {
        match (&self.provider, &self.query) {
            (Some(provider), Some(query)) => provider.matching(query.prefix()),
            _ => Vec::new(),
        }
    }

    pub fn diagnostic(&self) -> Option<&str> {
        self.error
            .as_deref()
            .or_else(|| self.provider.as_ref().and_then(Provider::diagnostic))
    }

    pub fn requested_rows(&self, limit: usize) -> u16 {
        let count = self.candidates().len();
        (count + usize::from(self.diagnostic().is_some()))
            .max(1)
            .min(limit) as u16
    }

    pub fn selected(&self) -> Option<usize> {
        (self.selected < self.candidates().len()).then_some(self.selected)
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn navigate(&mut self, previous: bool) {
        let count = self.candidates().len();
        if count == 0 {
            return;
        }
        self.selected = if previous {
            self.selected.saturating_sub(1)
        } else {
            (self.selected + 1).min(count - 1)
        };
        self.chosen = true;
    }

    pub fn reveal(&mut self, rows: u16) {
        let rows = usize::from(rows).saturating_sub(usize::from(self.diagnostic().is_some()));
        if rows == 0 {
            return;
        }
        if self.selected < self.top {
            self.top = self.selected;
        } else if self.selected >= self.top + rows {
            self.top = self.selected + 1 - rows;
        }
        self.top = self.top.min(self.candidates().len().saturating_sub(rows));
    }

    /// Resolve only the literal draft. Explicit candidate acceptance belongs to
    /// the file-prompt owner, independently of list rendering and selection.
    pub fn typed_path(&self) -> Option<PathBuf> {
        self.query
            .as_ref()
            .map(|query| query.directory().join(query.prefix()))
    }

    pub fn directory(&self) -> Option<&Path> {
        self.query.as_ref().map(Query::directory)
    }
}
