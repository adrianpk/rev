use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use super::entry::failure;
use super::{Entry, Tree};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreateKind {
    File,
    Directory,
}

/// Captures the chosen directory while a name prompt owns input. Publishing
/// consumes this request once and never replaces an existing filesystem entry.
pub struct Creation {
    directory: PathBuf,
    kind: CreateKind,
}

impl Creation {
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    pub fn kind(&self) -> CreateKind {
        self.kind
    }

    pub fn publish(self, name: &str) -> io::Result<PathBuf> {
        crate::file::name::validate(name)
            .map_err(|e| failure("create entry", &self.directory, e))?;
        directory(&self.directory)?;
        let path = self.directory.join(name);
        available(&path)?;
        let result = match self.kind {
            CreateKind::File => OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map(drop),
            CreateKind::Directory => fs::create_dir(&path),
        };
        result.map_err(|e| failure("create entry", &path, e))?;
        Ok(path)
    }
}

impl Tree {
    /// Prepare without creating anything. A file or symlink selection cannot
    /// silently change the requested location to a different directory.
    pub fn prepare_create(&self, kind: CreateKind) -> io::Result<Creation> {
        let entry = self
            .selected()
            .ok_or_else(|| io::Error::other("no tree selection"))?;
        directory(entry.path())?;
        Ok(Creation {
            directory: entry.path().to_path_buf(),
            kind,
        })
    }

    /// Observe a published child and select it. Errors describe observation,
    /// never undo or conceal the creation that already succeeded on disk.
    pub fn reveal_created(&mut self, path: &Path) -> io::Result<()> {
        let refreshed = self.refresh();
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("created entry has no parent"))?;
        self.selected = self
            .entries
            .iter()
            .position(|entry| entry.path() == parent)
            .ok_or_else(|| {
                failure(
                    "refresh created entry",
                    path,
                    io::Error::other("parent is no longer visible"),
                )
            })?;
        self.expand()?;
        self.selected = self
            .entries
            .iter()
            .position(|entry| entry.path() == path)
            .ok_or_else(|| {
                failure(
                    "refresh created entry",
                    path,
                    io::Error::other("entry is no longer visible"),
                )
            })?;
        refreshed
    }

    /// Opens/saves reconcile loaded parents only; collapsed branches remain
    /// lazy and the retained root and selection use the normal refresh policy.
    pub(crate) fn observe_path(&mut self, path: &Path) -> io::Result<()> {
        let loaded = self
            .entries
            .iter()
            .any(|entry| entry.directory && entry.expanded && Some(entry.path()) == path.parent());
        if loaded {
            self.refresh()?;
        }
        Ok(())
    }
}

fn directory(path: &Path) -> io::Result<()> {
    let entry = Entry::observe(path.to_path_buf(), 0)?;
    if !entry.is_directory() {
        return Err(failure(
            "create entry",
            path,
            io::Error::new(io::ErrorKind::InvalidInput, "select a directory"),
        ));
    }
    Ok(())
}

fn available(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(failure(
            "create entry",
            path,
            io::Error::new(io::ErrorKind::AlreadyExists, "entry already exists"),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(failure("inspect destination", path, error)),
    }
}
