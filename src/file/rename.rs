use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Captured entry identity, never a symlink's referent. A name prompt can be
/// canceled by dropping this request without changing disk or buffer metadata.
pub struct Rename {
    source: PathBuf,
    device: u64,
    inode: u64,
    directory: bool,
    symlink: bool,
}

impl Rename {
    pub fn capture(source: &Path) -> io::Result<Self> {
        let parent = source
            .parent()
            .ok_or_else(|| failure(source, io::Error::other("cannot rename filesystem root")))?;
        let name = source
            .file_name()
            .ok_or_else(|| failure(source, io::Error::other("entry has no name")))?;
        let source = fs::canonicalize(parent)
            .map_err(|e| failure(source, e))?
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
        })
    }

    pub fn source(&self) -> &Path {
        &self.source
    }
    pub(crate) fn is_directory(&self) -> bool {
        self.directory
    }
    pub(crate) fn is_symlink(&self) -> bool {
        self.symlink
    }

    pub(crate) fn destination(&self, name: &str) -> io::Result<PathBuf> {
        super::name::validate(name).map_err(|e| failure(&self.source, e))?;
        let target = self.source.with_file_name(name);
        self.validate(&target)?;
        Ok(target)
    }

    pub(crate) fn publish(&self, target: &Path) -> io::Result<()> {
        self.validate(target)?;
        fs::rename(&self.source, target).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "rename {} to {}: {e}",
                    self.source.display(),
                    target.display()
                ),
            )
        })
    }

    fn validate(&self, target: &Path) -> io::Result<()> {
        let metadata = fs::symlink_metadata(&self.source).map_err(|e| failure(&self.source, e))?;
        if metadata.dev() != self.device || metadata.ino() != self.inode {
            return Err(failure(
                &self.source,
                io::Error::other("selected entry changed after rename capture"),
            ));
        }
        match fs::symlink_metadata(target) {
            Ok(_) => Err(failure(
                target,
                io::Error::new(io::ErrorKind::AlreadyExists, "destination already exists"),
            )),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(failure(target, error)),
        }
    }
}

pub(crate) fn failure(path: &Path, error: io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("rename {}: {error}", path.display()))
}

/// Disk publication is complete even when later observation reports a warning.
pub struct Renamed {
    pub(crate) source: PathBuf,
    pub(crate) target: PathBuf,
    pub(crate) warning: Option<String>,
}
impl Renamed {
    pub fn source(&self) -> &Path {
        &self.source
    }
    pub fn target(&self) -> &Path {
        &self.target
    }
    pub fn warning(&self) -> Option<&str> {
        self.warning.as_deref()
    }
}
