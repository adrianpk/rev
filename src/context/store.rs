use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use rustix::fs::{Mode, OFlags};

use super::*;
use crate::file::{Files, Output, temporary::Temporary};

/// Successful replacement remains published even if later durability or
/// observation fails. Callers must activate/reconcile this exact candidate and
/// report the warning, rather than retrying creation under a new identity.
#[derive(Debug)]
pub struct Publication {
    pub catalog: Catalog,
    pub durability_warning: Option<&'static str>,
}

pub struct Store {
    path: PathBuf,
}
impl Store {
    pub fn at_state(roots: &crate::config::Roots) -> Result<(Self, Catalog), Error> {
        Self::open(&roots.state.join("contexts.toml"))
    }
    /// Read-only open. Missing state stays missing; malformed/unsupported input
    /// is preserved and rejected, never replaced with an empty catalog.
    pub fn open(path: &Path) -> Result<(Self, Catalog), Error> {
        NativePath::absolute(path)?;
        if path.file_name().is_none_or(|name| name != "contexts.toml") {
            return Err(Error::Invalid("catalog filename"));
        }
        let store = Self { path: path.into() };
        let catalog = match store.parent(false)? {
            None => Catalog::default(),
            Some(parent) => decode(read(&anchor(&parent).join("contexts.toml"))?)?,
        };
        Ok((store, catalog))
    }
    pub fn publish(&self, prepared: &Prepared) -> Result<Publication, Error> {
        self.publish_with(prepared, &mut Files)
    }
    /// Reuse the native source-save boundary for deterministic faults. Its
    /// replace contract guarantees that an error precedes the commit point.
    pub fn publish_with(
        &self,
        prepared: &Prepared,
        output: &mut impl Output,
    ) -> Result<Publication, Error> {
        let parent = self
            .parent(true)?
            .ok_or(Error::Unavailable("cannot create state directory"))?;
        let base = anchor(&parent);
        let lock = lock(&base.join("contexts.toml.lock"))?;
        self.check_parent(&parent)?;
        let path = base.join("contexts.toml");
        let before = read(&path)?;
        let current = decode(before.clone())?;
        if current != prepared.expected {
            return Err(Error::Conflict("stale catalog, record or session"));
        }
        // Canonical identities are observed again while publication is owned.
        prepared.check_roots()?;
        prepared.candidate.validate(true)?;
        let mut temporary =
            Temporary::new(&base).map_err(|_| Error::Unavailable("cannot stage catalog"))?;
        temporary
            .file
            .write_all(&prepared.bytes)
            .and_then(|()| output.flush(&mut temporary.file))
            .and_then(|()| output.sync_file(&temporary.file))
            .map_err(|_| Error::Unavailable("cannot synchronize staged catalog"))?;
        self.check_parent(&parent)?;
        lock.check(&base.join("contexts.toml.lock"))?;
        prepared.check_roots()?;
        if read(&path)? != before {
            return Err(Error::Conflict("catalog changed before replacement"));
        }
        output
            .replace(&temporary.path, &path)
            .map_err(|_| Error::Unavailable("cannot replace catalog"))?;
        temporary.disarm();
        let durable = output
            .sync_directory(&parent)
            .and_then(|()| output.observe(&path).map(|_| ()))
            .is_ok()
            && read(&path).is_ok_and(|bytes| bytes.as_deref() == Some(prepared.bytes.as_slice()))
            && self.check_parent(&parent).is_ok();
        Ok(Publication { catalog: prepared.candidate.clone(), durability_warning: (!durable)
            .then_some("Catalog was published; directory durability or final observation could not be confirmed") })
    }

    // Walk directory descriptors with NOFOLLOW, including intermediate state
    // components. A substituted symlink cannot redirect staging or publication.
    fn parent(&self, create: bool) -> Result<Option<File>, Error> {
        let parent = self.path.parent().ok_or(Error::Invalid("state parent"))?;
        let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
        let mut directory: File = rustix::fs::open("/", flags, Mode::empty())
            .map_err(|_| Error::Unavailable("cannot open filesystem root"))?
            .into();
        for component in parent.components() {
            let std::path::Component::Normal(name) = component else {
                continue;
            };
            let child = match rustix::fs::openat(&directory, name, flags, Mode::empty()) {
                Ok(child) => child,
                Err(rustix::io::Errno::NOENT) if !create => return Ok(None),
                Err(rustix::io::Errno::NOENT) => {
                    match rustix::fs::mkdirat(&directory, name, Mode::from_raw_mode(0o700)) {
                        Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                        Err(_) => return Err(Error::Unavailable("cannot create state directory")),
                    }
                    rustix::fs::openat(&directory, name, flags, Mode::empty())
                        .map_err(|_| Error::Unavailable("state directory changed"))?
                }
                Err(_) => return Err(Error::Unavailable("cannot open ordinary state directory")),
            };
            directory = child.into();
        }
        Ok(Some(directory))
    }
    fn check_parent(&self, directory: &File) -> Result<(), Error> {
        let current = self
            .parent(false)?
            .ok_or(Error::Unavailable("state directory disappeared"))?;
        let a = directory
            .metadata()
            .map_err(|_| Error::Unavailable("cannot inspect state directory"))?;
        let b = current
            .metadata()
            .map_err(|_| Error::Unavailable("cannot inspect state directory"))?;
        if a.dev() != b.dev() || a.ino() != b.ino() {
            return Err(Error::Unavailable("state directory identity changed"));
        }
        Ok(())
    }
}
fn decode(bytes: Option<Vec<u8>>) -> Result<Catalog, Error> {
    match bytes {
        Some(bytes) => Catalog::decode(&bytes),
        None => Ok(Catalog::default()),
    }
}
fn anchor(directory: &File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()))
}
fn flags() -> i32 {
    (OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC).bits() as i32
}
struct Lock(File);
impl Lock {
    fn check(&self, path: &Path) -> Result<(), Error> {
        let opened = self
            .0
            .metadata()
            .map_err(|_| Error::Unavailable("cannot inspect publication lock"))?;
        let current = fs::symlink_metadata(path)
            .map_err(|_| Error::Unavailable("publication lock disappeared"))?;
        if !current.is_file() || opened.dev() != current.dev() || opened.ino() != current.ino() {
            return Err(Error::Unavailable("publication lock identity changed"));
        }
        Ok(())
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
fn lock(path: &Path) -> Result<Lock, Error> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(flags())
        .open(path)
        .map_err(|_| Error::Unavailable("cannot open publication lock"))?;
    let opened = file
        .metadata()
        .map_err(|_| Error::Unavailable("cannot inspect publication lock"))?;
    let current = fs::symlink_metadata(path)
        .map_err(|_| Error::Unavailable("publication lock disappeared"))?;
    if !opened.is_file()
        || !current.is_file()
        || opened.dev() != current.dev()
        || opened.ino() != current.ino()
    {
        return Err(Error::Unavailable("publication lock identity changed"));
    }
    match file.try_lock() {
        Ok(()) => Ok(Lock(file)),
        Err(std::fs::TryLockError::WouldBlock) => Err(Error::Busy),
        Err(_) => Err(Error::Unavailable("cannot acquire publication lock")),
    }
}
fn read(path: &Path) -> Result<Option<Vec<u8>>, Error> {
    let mut file = match OpenOptions::new()
        .read(true)
        .custom_flags(flags())
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(Error::Unavailable("cannot read ordinary catalog")),
    };
    let before = file
        .metadata()
        .map_err(|_| Error::Unavailable("cannot inspect catalog"))?;
    if !before.is_file() {
        return Err(Error::Invalid("catalog file type"));
    }
    if before.len() > MAX_CATALOG_BYTES as u64 {
        return Err(Error::Capacity("encoded catalog"));
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    Read::by_ref(&mut file)
        .take(MAX_CATALOG_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Unavailable("cannot finish reading catalog"))?;
    if bytes.len() > MAX_CATALOG_BYTES {
        return Err(Error::Capacity("encoded catalog"));
    }
    let after = file
        .metadata()
        .map_err(|_| Error::Unavailable("cannot finish inspecting catalog"))?;
    let current =
        fs::symlink_metadata(path).map_err(|_| Error::Unavailable("catalog disappeared"))?;
    if before.len() != after.len()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
        || !current.is_file()
        || current.dev() != after.dev()
        || current.ino() != after.ino()
    {
        return Err(Error::Conflict("catalog changed during read"));
    }
    Ok(Some(bytes))
}
