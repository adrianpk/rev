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
        inspect_lock(&base.join("library.toml.lock"))?;
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
        validate_candidate(record.as_ref(), name, expected)?;
        let guard = self.publication_guard()?;
        let before = read(&guard.path)?;
        let mut fresh = Catalog::decode(before.as_deref())?;
        revise_catalog(&mut fresh, record.as_ref(), name, expected)?;
        let bytes = fresh.encode()?;
        let observed = guard.replace(self, before.as_deref(), &bytes, output)?;
        // Replacement is the native commit point. Retain this candidate even
        // when the later directory-sync/observation cannot prove durability.
        Ok(Publication {
            record,
            warning: !observed,
        })
    }

    fn publication_guard(&self) -> Result<PublicationGuard, Error> {
        let parent = self
            .parent(true)?
            .ok_or(Error::Unavailable("cannot create macros directory"))?;
        self.check_private(&parent)?;
        let base = anchor(&parent);
        let lock = lock(&base.join("library.toml.lock"))?;
        self.check_parent(&parent)?;
        let path = base.join("library.toml");
        Ok(PublicationGuard {
            lock,
            directory: parent,
            base,
            path,
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
            let Some(child) = directory_child(&directory, name, flags, create)? else {
                return Ok(None);
            };
            directory = child;
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
    (OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC)
        .bits()
        .cast_signed()
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
    check_lock_file(&file, path)?;
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
    let before = library_metadata(&file)?;
    let bytes = library_bytes(&mut file, &before)?;
    check_library_read(&file, path, &before)?;
    Ok(Some(bytes))
}

// Field drop order releases the lock before closing the anchored directory.
struct PublicationGuard {
    lock: Lock,
    directory: File,
    base: PathBuf,
    path: PathBuf,
}

impl PublicationGuard {
    fn stage(&self, bytes: &[u8], output: &mut impl Output) -> Result<Temporary, Error> {
        let mut temporary =
            Temporary::new(&self.base).map_err(|_| Error::Unavailable("cannot stage library"))?;
        temporary
            .file
            .write_all(bytes)
            .and_then(|()| output.flush(&mut temporary.file))
            .and_then(|()| output.sync_file(&temporary.file))
            .map_err(|_| Error::Unavailable("cannot synchronize staged library"))?;
        Ok(temporary)
    }

    fn replace(
        &self,
        store: &Store,
        before: Option<&[u8]>,
        bytes: &[u8],
        output: &mut impl Output,
    ) -> Result<bool, Error> {
        let mut temporary = self.stage(bytes, output)?;
        store.check_parent(&self.directory)?;
        self.lock.check(&self.base.join("library.toml.lock"))?;
        if read(&self.path)?.as_deref() != before {
            return Err(Error::Conflict("library changed before replacement"));
        }
        output
            .replace(&temporary.path, &self.path)
            .map_err(|_| Error::Unavailable("cannot replace library"))?;
        temporary.disarm();
        let observed = output
            .sync_directory(&self.directory)
            .and_then(|()| output.observe(&self.path).map(|_| ()))
            .is_ok()
            && store.check_parent(&self.directory).is_ok()
            && read(&self.path).is_ok_and(|v| v.as_deref() == Some(bytes));
        Ok(observed)
    }
}

fn validate_candidate(
    record: Option<&Record>,
    name: &str,
    expected: Option<(&str, u64)>,
) -> Result<(), Error> {
    if let Some(candidate) = record {
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
    if let Some(record) = record {
        record.definition(0)?;
    }
    Ok(())
}

fn revise_catalog(
    catalog: &mut Catalog,
    record: Option<&Record>,
    name: &str,
    expected: Option<(&str, u64)>,
) -> Result<(), Error> {
    let index = catalog.macros.iter().position(|r| r.name == name);
    match (index, expected) {
        (None, None) => {}
        (Some(i), Some((id, revision)))
            if catalog.macros[i].id == id && catalog.macros[i].revision == revision => {}
        _ => {
            return Err(Error::Conflict(
                "saved name, identity or record revision changed",
            ));
        }
    }
    match (record, index) {
        (Some(record), Some(i)) => catalog.macros[i] = record.clone(),
        (Some(record), None) => catalog.macros.push(record.clone()),
        (None, Some(i)) => {
            catalog.macros.remove(i);
        }
        (None, None) => return Err(Error::Conflict("saved record disappeared")),
    }
    catalog.revision = catalog
        .revision
        .checked_add(1)
        .filter(|n| *n <= i64::MAX as u64)
        .ok_or(Error::Capacity("aggregate revision exhausted"))?;
    // Upgrade only at this explicit write, preserving every other record's
    // identity, revision and content. Reading never changes the aggregate.
    catalog.schema_version = catalog
        .schema_version
        .max(catalog.macros.iter().map(Record::schema).max().unwrap_or(1));
    Ok(())
}

fn inspect_lock(path: &Path) -> Result<(), Error> {
    // Read-only admission inspects an existing lock without creating one.
    // Symlink/nonregular entries cannot be deferred until a later write.
    match OpenOptions::new()
        .read(true)
        .custom_flags(flags())
        .open(path)
    {
        Ok(file)
            if file
                .metadata()
                .is_ok_and(|m| m.is_file() && m.mode() & 0o077 == 0) => {}
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound && fs::symlink_metadata(path).is_err() => {}
        _ => return Err(Error::Invalid("ordinary private library lock")),
    }
    Ok(())
}

fn directory_child(
    directory: &File,
    name: &std::ffi::OsStr,
    flags: OFlags,
    create: bool,
) -> Result<Option<File>, Error> {
    match rustix::fs::openat(directory, name, flags, Mode::empty()) {
        Ok(child) => Ok(Some(child.into())),
        Err(rustix::io::Errno::NOENT) if !create => Ok(None),
        Err(rustix::io::Errno::NOENT) => {
            match rustix::fs::mkdirat(directory, name, Mode::from_raw_mode(0o700)) {
                Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                Err(_) => return Err(Error::Unavailable("cannot create state directory")),
            }
            rustix::fs::openat(directory, name, flags, Mode::empty())
                .map(|child| Some(child.into()))
                .map_err(|_| Error::Unavailable("state directory changed"))
        }
        Err(_) => Err(Error::Unavailable("cannot open ordinary state directory")),
    }
}

fn check_lock_file(file: &File, path: &Path) -> Result<(), Error> {
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
    Ok(())
}

fn library_metadata(file: &File) -> Result<fs::Metadata, Error> {
    let before = file
        .metadata()
        .map_err(|_| Error::Unavailable("cannot inspect macro library"))?;
    if before.mode() & 0o077 != 0 || !before.is_file() {
        return Err(Error::Invalid("macro library file type"));
    }
    if before.len() > codec::LIBRARY_BYTES as u64 {
        return Err(Error::Capacity("encoded macro library"));
    }
    Ok(before)
}

fn library_bytes(file: &mut File, before: &fs::Metadata) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::with_capacity(
        usize::try_from(before.len()).expect("bounded encoded macro library size"),
    );
    Read::by_ref(file)
        .take(codec::LIBRARY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Unavailable("cannot finish reading macro library"))?;
    if bytes.len() > codec::LIBRARY_BYTES {
        return Err(Error::Capacity("encoded macro library"));
    }
    Ok(bytes)
}

fn check_library_read(file: &File, path: &Path, before: &fs::Metadata) -> Result<(), Error> {
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
    Ok(())
}
