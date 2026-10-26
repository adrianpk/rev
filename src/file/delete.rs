use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Captures the selected entry itself, including a symlink rather than its
/// referent. Dropping either confirmation stage leaves the filesystem intact.
pub struct Deletion {
    source: PathBuf,
    device: u64,
    inode: u64,
    directory: bool,
    symlink: bool,
    recursive: bool,
}

/// Recursive requires a second explicit decision before calling confirm again.
pub enum DeleteDecision {
    Ready(Deletion),
    Recursive(Deletion),
}

impl Deletion {
    pub fn capture(path: &Path) -> io::Result<Self> {
        let parent = path
            .parent()
            .ok_or_else(|| failure(path, io::Error::other("cannot delete filesystem root")))?;
        let name = path
            .file_name()
            .ok_or_else(|| failure(path, io::Error::other("entry has no name")))?;
        let source = fs::canonicalize(parent)
            .map_err(|e| failure(path, e))?
            .join(name);
        let metadata = fs::symlink_metadata(&source).map_err(|e| failure(&source, e))?;
        if !metadata.is_file() && !metadata.is_dir() && !metadata.is_symlink() {
            return Err(failure(
                &source,
                io::Error::other("expected a file, directory or symlink"),
            ));
        }
        Ok(Self {
            source,
            device: metadata.dev(),
            inode: metadata.ino(),
            directory: metadata.is_dir(),
            symlink: metadata.is_symlink(),
            recursive: false,
        })
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn message(&self) -> String {
        if self.recursive {
            format!(
                "Directory contains children; recursively delete {}? [y/n]",
                self.source.display()
            )
        } else {
            format!("Delete {}? [y/n]", self.source.display())
        }
    }

    /// One affirmative answer consumes one stage. An empty directory never
    /// acquires recursive authority if children appear after this inspection.
    pub fn confirm(mut self) -> io::Result<DeleteDecision> {
        self.validate()?;
        if self.directory && !self.recursive && self.has_children()? {
            self.recursive = true;
            return Ok(DeleteDecision::Recursive(self));
        }
        Ok(DeleteDecision::Ready(self))
    }

    fn has_children(&self) -> io::Result<bool> {
        let mut entries = fs::read_dir(&self.source).map_err(|e| failure(&self.source, e))?;
        entries
            .next()
            .transpose()
            .map(|entry| entry.is_some())
            .map_err(|e| failure(&self.source, e))
    }

    pub(crate) fn includes(&self, path: &Path) -> bool {
        !self.symlink && (path == self.source || (self.directory && path.starts_with(&self.source)))
    }

    pub(crate) fn publish(&self) -> io::Result<()> {
        self.validate()?;
        let result = if self.recursive {
            fs::remove_dir_all(&self.source)
        } else if self.directory {
            fs::remove_dir(&self.source)
        } else {
            fs::remove_file(&self.source)
        };
        result.map_err(|e| failure(&self.source, e))
    }

    fn validate(&self) -> io::Result<()> {
        let metadata = fs::symlink_metadata(&self.source).map_err(|e| failure(&self.source, e))?;
        if metadata.dev() != self.device || metadata.ino() != self.inode {
            return Err(failure(
                &self.source,
                io::Error::other("selected entry changed after delete capture"),
            ));
        }
        Ok(())
    }
}

fn failure(path: &Path, error: io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("delete {}: {error}", path.display()))
}

/// Deletion can fail after removing some children. The error records that
/// outcome; missing visiting paths are reconciled even after partial failure.
pub struct Deleted {
    pub(crate) source: PathBuf,
    pub(crate) error: Option<io::Error>,
    pub(crate) warning: Option<String>,
}
impl Deleted {
    pub fn source(&self) -> &Path {
        &self.source
    }
    pub fn error(&self) -> Option<&io::Error> {
        self.error.as_ref()
    }
    pub fn warning(&self) -> Option<&str> {
        self.warning.as_deref()
    }
    pub fn is_complete(&self) -> bool {
        self.error.is_none()
    }
}
