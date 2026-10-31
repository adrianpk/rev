use super::*;
use crate::bookmark::store::Store;
use std::path::PathBuf;

/// Embedded sessions discover no user files; an explicit configured path opts
/// in. Executable startup supplies the XDG default without reading the store.
#[derive(Default)]
pub(in crate::session) struct Persistence {
    path: Option<PathBuf>,
    store: Option<Store>,
    error: Option<String>,
    attempted: bool,
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
