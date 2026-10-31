use super::*;
use crate::bookmark::store::Store;
use std::path::PathBuf;

/// Embedded sessions discover no user files; an explicit configured path opts
/// in. Executable startup supplies the XDG default without reading the store.
#[derive(Default)]
pub(in crate::session) struct Persistence {
    pub(super) path: Option<PathBuf>,
    store: Option<Store>,
    pub(super) error: Option<String>,
    pub(super) attempted: bool,
}
impl Persistence {
    pub(in crate::session) fn configured(path: Option<PathBuf>) -> Self {
        Self {
            path,
            ..Self::default()
        }
    }
}
impl Session {
    pub(in crate::session) fn startup_bookmarks(&mut self) {
        if self.bookmark_storage.path.is_none() {
            match crate::config::Roots::from_env() {
                Ok(roots) => self.bookmark_storage.path = Some(roots.data.join("bookmarks.toml")),
                Err(error) => self.bookmark_storage.error = Some(error.to_string()),
            }
        }
    }
    pub(in crate::session) fn ensure_bookmarks(&mut self) -> Result<(), String> {
        if self.bookmark_storage.attempted {
            return self.bookmark_storage.error.clone().map_or(Ok(()), Err);
        }
        self.bookmark_storage.attempted = true;
        if let Some(error) = &self.bookmark_storage.error {
            return Err(error.clone());
        }
        let Some(path) = &self.bookmark_storage.path else {
            return Ok(());
        };
        let limit = self.buffers.snapshot().settings().bookmarks.max_entries;
        let loaded = Store::open(path, limit).and_then(|(store, records)| {
            self.buffers.bookmarks_mut().admit(records)?;
            Ok(store)
        });
        match loaded {
            Ok(store) => {
                self.bookmark_storage.store = Some(store);
                Ok(())
            }
            Err(error) => {
                let message = format!(
                    "Bookmarks: {error}; persistence blocked; use bookmark-reload after correction"
                );
                self.bookmark_storage.error = Some(message.clone());
                Err(message)
            }
        }
    }
}

pub(in crate::session) struct Reload {
    store: Store,
    records: Vec<crate::bookmark::store::Stored>,
}
impl Session {
    pub(in crate::session) fn save_bookmarks(&mut self) -> Result<(), String> {
        self.ensure_bookmarks()?;
        if self.bookmark_storage.store.is_none() {
            return Err("Bookmark storage is not configured by this session's host".into());
        }
        // Refresh only checked live anchors; stale/unresolved records retain
        // their last usable evidence. This never saves a source document.
        {
            let mut records = self.buffers.bookmarks_mut();
            for buffer in self.buffers.iter() {
                records
                    .checkpoint(buffer, false)
                    .map_err(|e| e.to_string())?;
            }
        }
        let records = self.buffers.bookmarks();
        self.bookmark_storage
            .store
            .as_mut()
            .ok_or("Bookmark storage unavailable")?
            .write(&records)?;
        drop(records);
        self.buffers.bookmarks_mut().saved();
        Ok(())
    }

    pub(in crate::session) fn reload_bookmarks(&mut self) -> Action {
        let Some(path) = &self.bookmark_storage.path else {
            return self.error("Bookmark storage is not configured by this session's host");
        };
        let limit = self.buffers.snapshot().settings().bookmarks.max_entries;
        let (store, records) = match Store::open(path, limit) {
            Ok(candidate) => candidate,
            Err(error) => return self.error(&error),
        };
        let candidate = Reload { store, records };
        if self.buffers.bookmarks().is_dirty() {
            if let Err(error) = self
                .input
                .decision("Discard unsaved bookmarks and reload? [y/n]")
            {
                return self.error(&error.to_string());
            }
            self.pending = Some(Pending::BookmarkReload(Box::new(candidate)));
            return self
                .message("Discard unsaved bookmarks and reload? y=reload, n=retain; C-g=cancel");
        }
        self.bookmark_reload_answer(candidate, true)
    }

    pub(in crate::session) fn bookmark_reload_answer(
        &mut self,
        candidate: Reload,
        accepted: bool,
    ) -> Action {
        if !accepted {
            return self.message("Bookmark reload canceled; memory and disk retained");
        }
        let result = candidate
            .store
            .guard()
            .and_then(|()| self.buffers.bookmarks_mut().admit(candidate.records));
        if let Err(error) = result {
            return self.error(&error);
        }
        self.bookmark_storage.store = Some(candidate.store);
        self.bookmark_storage.error = None;
        self.bookmark_storage.attempted = true;
        self.message("Bookmarks reloaded")
    }

    pub(in crate::session) fn bookmark_change_message(&mut self, message: &str) -> Action {
        if self.buffers.snapshot().settings().bookmarks.save
            == crate::config::BookmarkSave::OnChange
            && (self.bookmark_storage.path.is_some() || self.bookmark_storage.error.is_some())
            && let Err(error) = self.save_bookmarks()
        {
            return self.error(&format!(
                "{message}; {error}; bookmark changes remain unsaved"
            ));
        }
        self.message(message)
    }
}
