use super::{Catalog, Error, Record, Saved, codec};
use crate::file::{Files, Output, temporary::Temporary};
use rustix::fs::{Mode, OFlags};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

pub(crate) struct Publication {
    pub record: Option<Record>,
    pub warning: bool,
}
pub(crate) struct Store {
    path: PathBuf,
}
impl Store {
    pub fn at_state(roots: &crate::config::Roots) -> Result<Self, Error> {
        if !roots.state.is_absolute()
            || roots
                .state
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(Error::Invalid("absolute state root"));
        }
        Ok(Self {
            path: roots.state.join("macros/library.toml"),
        })
    }
    pub fn load(&self) -> Result<Catalog, Error> {
        let Some(parent) = self.parent(false)? else {
            return Ok(Catalog::default());
        };
        self.check_private(&parent)?;
        let base = anchor(&parent);
        // Read-only admission inspects an existing lock without creating one.
        // Symlink/nonregular entries cannot be deferred until a later write.
        match OpenOptions::new()
            .read(true)
            .custom_flags(flags())
            .open(base.join("library.toml.lock"))
        {
            Ok(file)
                if file
                    .metadata()
                    .is_ok_and(|m| m.is_file() && m.mode() & 0o077 == 0) => {}
            Err(e)
                if e.kind() == std::io::ErrorKind::NotFound
                    && fs::symlink_metadata(base.join("library.toml.lock")).is_err() => {}
            _ => return Err(Error::Invalid("ordinary private library lock")),
        }
        let bytes = read(&base.join("library.toml"))?;
        self.check_parent(&parent)?;
        Catalog::decode(bytes.as_deref())
    }
    pub fn save(&self, mut record: Record, expected: Option<&Saved>) -> Result<Publication, Error> {
        record.revision = expected.map_or(Ok(1), |s| {
            s.revision
                .checked_add(1)
                .filter(|n| *n <= i64::MAX as u64)
                .ok_or(Error::Capacity("record revision exhausted"))
        })?;
        self.publish(
            Some(record.clone()),
            &record.name,
            expected.map(|s| (s.id.as_str(), s.revision)),
            &mut Files,
        )
    }
    pub fn delete(&self, name: &str, id: &str, revision: u64) -> Result<Publication, Error> {
        self.publish(None, name, Some((id, revision)), &mut Files)
    }
    pub(super) fn publish(
        &self,
        record: Option<Record>,
        name: &str,
        expected: Option<(&str, u64)>,
        output: &mut impl Output,
    ) -> Result<Publication, Error> {
        if let Some(candidate) = &record {
            let valid = match expected {
                Some((id, revision)) => {
                    candidate.id == id && revision.checked_add(1) == Some(candidate.revision)
                }
                None => candidate.revision == 1,
            };
            if candidate.name != name || !valid {
                return Err(Error::Invalid("candidate identity/revision"));
            }
        }
        // Candidate bounds are checked before any directory/lock is created.
        if let Some(record) = &record {
            record.definition(0)?;
        }
        let parent = self
            .parent(true)?
            .ok_or(Error::Unavailable("cannot create macros directory"))?;
        self.check_private(&parent)?;
        let base = anchor(&parent);
        let lock = lock(&base.join("library.toml.lock"))?;
        self.check_parent(&parent)?;
        let path = base.join("library.toml");
        let before = read(&path)?;
        let mut fresh = Catalog::decode(before.as_deref())?;
        let index = fresh.macros.iter().position(|r| r.name == name);
        match (index, expected) {
            (None, None) => {}
            (Some(i), Some((id, revision)))
                if fresh.macros[i].id == id && fresh.macros[i].revision == revision => {}
            _ => {
                return Err(Error::Conflict(
                    "saved name, identity or record revision changed",
                ));
            }
        }
        match (&record, index) {
            (Some(record), Some(i)) => fresh.macros[i] = record.clone(),
            (Some(record), None) => fresh.macros.push(record.clone()),
            (None, Some(i)) => {
                fresh.macros.remove(i);
            }
            (None, None) => return Err(Error::Conflict("saved record disappeared")),
        }
        fresh.revision = fresh
            .revision
            .checked_add(1)
            .filter(|n| *n <= i64::MAX as u64)
            .ok_or(Error::Capacity("aggregate revision exhausted"))?;
        let bytes = fresh.encode()?;
        let mut temporary =
            Temporary::new(&base).map_err(|_| Error::Unavailable("cannot stage library"))?;
        temporary
            .file
            .write_all(&bytes)
            .and_then(|()| output.flush(&mut temporary.file))
            .and_then(|()| output.sync_file(&temporary.file))
            .map_err(|_| Error::Unavailable("cannot synchronize staged library"))?;
        self.check_parent(&parent)?;
        lock.check(&base.join("library.toml.lock"))?;
        if read(&path)? != before {
            return Err(Error::Conflict("library changed before replacement"));
        }
        output
            .replace(&temporary.path, &path)
            .map_err(|_| Error::Unavailable("cannot replace library"))?;
        temporary.disarm();
        let observed = output
            .sync_directory(&parent)
            .and_then(|()| output.observe(&path).map(|_| ()))
            .is_ok()
            && self.check_parent(&parent).is_ok()
            && read(&path).is_ok_and(|v| v.as_deref() == Some(bytes.as_slice()));
        // Replacement is the native commit point. Retain this candidate even
        // when the later directory-sync/observation cannot prove durability.
        Ok(Publication {
            record,
            warning: !observed,
        })
    }
    fn check_private(&self, parent: &File) -> Result<(), Error> {
        if parent
            .metadata()
            .map_err(|_| Error::Unavailable("cannot inspect macros directory"))?
            .mode()
            & 0o077
            != 0
        {
            return Err(Error::Invalid("macros directory must be private (0700)"));
        }
        Ok(())
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
    if opened.mode() & 0o077 != 0
        || !opened.is_file()
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
        Err(_) => return Err(Error::Unavailable("cannot read ordinary macro library")),
    };
    let before = file
        .metadata()
        .map_err(|_| Error::Unavailable("cannot inspect macro library"))?;
    if before.mode() & 0o077 != 0 || !before.is_file() {
        return Err(Error::Invalid("macro library file type"));
    }
    if before.len() > codec::LIBRARY_BYTES as u64 {
        return Err(Error::Capacity("encoded macro library"));
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    Read::by_ref(&mut file)
        .take(codec::LIBRARY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Unavailable("cannot finish reading macro library"))?;
    if bytes.len() > codec::LIBRARY_BYTES {
        return Err(Error::Capacity("encoded macro library"));
    }
    let after = file
        .metadata()
        .map_err(|_| Error::Unavailable("cannot finish inspecting macro library"))?;
    let current =
        fs::symlink_metadata(path).map_err(|_| Error::Unavailable("macro library disappeared"))?;
    if before.len() != after.len()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
        || !current.is_file()
        || current.dev() != after.dev()
        || current.ino() != after.ino()
    {
        return Err(Error::Conflict("macro library changed during read"));
    }
    Ok(Some(bytes))
}
