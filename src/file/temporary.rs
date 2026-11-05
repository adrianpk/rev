use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::Output;

static NEXT: AtomicU64 = AtomicU64::new(0);

pub(crate) struct Temporary {
    pub file: File,
    pub path: PathBuf,
    armed: bool,
}

impl Temporary {
    pub fn new(parent: &Path) -> io::Result<Self> {
        for _ in 0..32 {
            let path = candidate(parent)?;
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
            {
                Ok(file) => {
                    return Ok(Self {
                        file,
                        path,
                        armed: true,
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "temporary file collision limit reached",
        ))
    }

    /// Create an exclusively owned hard-link staging name without first
    /// unlinking a placeholder. Existing candidates never become cleanup rights.
    pub(crate) fn link(parent: &Path, source: &Path, input: &File) -> io::Result<Self> {
        let file = input.try_clone()?;
        for _ in 0..32 {
            let path = candidate(parent)?;
            match fs::hard_link(source, &path) {
                Ok(()) => {
                    return Ok(Self {
                        file,
                        path,
                        armed: true,
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "hard-link staging collision limit reached",
        ))
    }
    pub fn disarm(&mut self) {
        self.armed = false;
    }

    pub fn failed(mut self, error: io::Error, output: &mut impl Output) -> io::Error {
        match output.remove(&self.path) {
            Ok(()) => {
                self.disarm();
                error
            }
            Err(cleanup) => io::Error::new(
                error.kind(),
                format!(
                    "{error}; remove temporary output {:?}: {cleanup}",
                    self.path
                ),
            ),
        }
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        if self.armed {
            // Explicit failure paths report cleanup errors; Drop makes one last
            // best-effort attempt for this exclusively created owned file.
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn candidate(parent: &Path) -> io::Result<PathBuf> {
    let id = NEXT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map_err(|_| io::Error::other("temporary file identities are exhausted"))?;
    Ok(parent.join(format!(".rev-save-{:x}-{id:016x}", std::process::id())))
}

/// A private operation workspace owns only its explicitly declared producer
/// filenames. Unexpected entries are preserved, including during Drop.
pub(crate) struct Directory {
    root: PathBuf,
    directory: File,
    allowed: Vec<std::ffi::OsString>,
    owned: std::sync::atomic::AtomicBool,
}
impl Directory {
    pub fn new(parent: &Path, allowed: Vec<std::ffi::OsString>) -> io::Result<Self> {
        use std::os::unix::fs::DirBuilderExt;
        if allowed.iter().any(|name| {
            let mut parts = Path::new(name).components();
            !matches!(parts.next(), Some(std::path::Component::Normal(_))) || parts.next().is_some()
        }) {
            return Err(io::Error::other(
                "Workspace producer names must be ordinary single components",
            ));
        }
        let parent = fs::canonicalize(parent)?;
        for _ in 0..32 {
            let root = candidate(&parent)?;
            match fs::DirBuilder::new().mode(0o700).create(&root) {
                Ok(()) => {
                    let flags = rustix::fs::OFlags::DIRECTORY | rustix::fs::OFlags::NOFOLLOW;
                    let directory = OpenOptions::new()
                        .read(true)
                        .custom_flags(i32::try_from(flags.bits()).expect("directory flags"))
                        .open(&root)?;
                    return Ok(Self {
                        root,
                        directory,
                        allowed,
                        owned: std::sync::atomic::AtomicBool::new(true),
                    });
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::other(
            "Private workspace collision limit reached",
        ))
    }
    pub fn entry(&self, name: &str) -> io::Result<PathBuf> {
        use std::os::fd::AsRawFd;
        if !self.allowed.iter().any(|n| n == name) {
            return Err(io::Error::other("Undeclared workspace producer name"));
        }
        Ok(PathBuf::from(format!(
            "/proc/{}/fd/{}",
            std::process::id(),
            self.directory.as_raw_fd()
        ))
        .join(name))
    }
    pub fn anchor(&self) -> PathBuf {
        use std::os::fd::AsRawFd;
        PathBuf::from(format!(
            "/proc/{}/fd/{}",
            std::process::id(),
            self.directory.as_raw_fd()
        ))
    }
    pub fn read(&self, name: &str, limit: usize) -> io::Result<Vec<u8>> {
        use std::io::Read;
        let path = self.entry(name)?;
        let flags = rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK;
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(i32::try_from(flags.bits()).expect("file flags"))
            .open(path)?;
        let before = file.metadata()?;
        if !before.is_file() || before.len() > limit as u64 {
            return Err(io::Error::other(
                "Workspace output exceeds complete ordinary-file admission",
            ));
        }
        let mut bytes = Vec::with_capacity(before.len() as usize);
        file.by_ref()
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes)?;
        use std::os::unix::fs::MetadataExt;
        let after = file.metadata()?;
        if bytes.len() > limit
            || before.len() != after.len()
            || before.mode() != after.mode()
            || before.mtime() != after.mtime()
            || before.ctime() != after.ctime()
            || before.mtime_nsec() != after.mtime_nsec()
            || before.ctime_nsec() != after.ctime_nsec()
        {
            return Err(io::Error::other(
                "Workspace output changed during bounded read",
            ));
        }
        Ok(bytes)
    }
    pub fn clean(&self) -> io::Result<()> {
        use std::os::unix::fs::MetadataExt;
        if !self.owned.load(Ordering::Acquire) {
            return Ok(());
        }
        let expected = self.directory.metadata()?;
        let actual = fs::symlink_metadata(&self.root)?;
        if actual.dev() != expected.dev()
            || actual.ino() != expected.ino()
            || !actual.is_dir()
            || actual.file_type().is_symlink()
        {
            return Err(io::Error::other(
                "Private workspace identity changed; replacement preserved",
            ));
        }
        let mut names = Vec::new();
        for entry in fs::read_dir(self.anchor())? {
            let entry = entry?;
            if names.len() == self.allowed.len()
                || !self.allowed.contains(&entry.file_name())
                || !entry.file_type()?.is_file()
            {
                return Err(io::Error::other("Unknown workspace output preserved"));
            }
            names.push(entry.file_name());
        }
        for name in names {
            fs::remove_file(self.anchor().join(name))?;
        }
        fs::remove_dir(&self.root)?;
        self.owned.store(false, Ordering::Release);
        Ok(())
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = self.clean();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::Fixture;

    // Cleanup removes only declared ordinary producer files. Unexpected entries
    // or substituted paths preserve all bytes and cannot grant deletion rights.
    #[test]
    fn workspace_identity() {
        for variant in ["unknown", "symlink", "replacement", "complete"] {
            let fixture = Fixture::new();
            let workspace = Directory::new(&fixture.0, vec!["index".into()]).unwrap();
            let root = fs::canonicalize(workspace.anchor()).unwrap();
            fs::write(workspace.entry("index").unwrap(), b"owned").unwrap();
            assert_eq!(workspace.read("index", 5).unwrap(), b"owned");
            assert!(workspace.read("index", 4).is_err());
            match variant {
                "unknown" => {
                    fs::write(root.join("unknown"), b"preserved").unwrap();
                }
                "symlink" => {
                    fs::remove_file(root.join("index")).unwrap();
                    std::os::unix::fs::symlink("unrelated", root.join("index")).unwrap();
                }
                "replacement" => {
                    fs::rename(&root, fixture.0.join("retained")).unwrap();
                    fs::create_dir(&root).unwrap();
                }
                _ => {}
            }
            let cleaned = workspace.clean();
            drop(workspace);
            if variant == "complete" {
                assert!(cleaned.is_ok());
                assert!(!root.exists());
            } else {
                assert!(cleaned.is_err());
                assert!(root.exists());
            }
            if variant == "unknown" {
                assert_eq!(fs::read(root.join("index")).unwrap(), b"owned");
                assert_eq!(fs::read(root.join("unknown")).unwrap(), b"preserved");
            }
            if variant == "replacement" {
                assert_eq!(
                    fs::read(fixture.0.join("retained/index")).unwrap(),
                    b"owned"
                );
            }
        }
    }
}
