//! Descriptor-anchored native state IO, following the macro-library owner's
//! NOFOLLOW walk, nonblocking OS lock and checked replacement pattern.
use super::{Store, codec, conflict, error, invalid, limit, unavailable};
use rev_plugin_sdk::rpc::{ErrorKind, RpcError};
use rustix::fs::{Mode, OFlags};
use std::{
    fs::{self, File, OpenOptions},
    io::Read,
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
};
impl Store {
    // Walk directory descriptors with NOFOLLOW, including intermediate state
    // components. A substituted symlink cannot redirect staging or publication.
    pub(super) fn parent(&self, create: bool) -> Result<Option<File>, RpcError> {
        let parent = self.path.parent().ok_or(invalid("state parent"))?;
        let flags = directory_flags();
        let mut directory: File = rustix::fs::open("/", flags, Mode::empty())
            .map_err(|_| unavailable("cannot open filesystem root"))?
            .into();
        for component in parent.components() {
            let std::path::Component::Normal(name) = component else {
                continue;
            };
            let Some(child) = open_directory(&directory, name, create)? else {
                return Ok(None);
            };
            directory = child;
        }
        Ok(Some(directory))
    }
    pub(super) fn check_parent(&self, directory: &File) -> Result<(), RpcError> {
        let current = self
            .parent(false)?
            .ok_or(unavailable("state directory disappeared"))?;
        let a = directory
            .metadata()
            .map_err(|_| unavailable("cannot inspect state directory"))?;
        let b = current
            .metadata()
            .map_err(|_| unavailable("cannot inspect state directory"))?;
        if a.dev() != b.dev() || a.ino() != b.ino() {
            return Err(unavailable("state directory identity changed"));
        }
        Ok(())
    }
}
fn directory_flags() -> OFlags {
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
}
fn open_directory(
    directory: &File,
    name: &std::ffi::OsStr,
    create: bool,
) -> Result<Option<File>, RpcError> {
    let flags = directory_flags();
    let child = match rustix::fs::openat(directory, name, flags, Mode::empty()) {
        Ok(child) => child,
        Err(rustix::io::Errno::NOENT) if !create => return Ok(None),
        Err(rustix::io::Errno::NOENT) => {
            match rustix::fs::mkdirat(directory, name, Mode::from_raw_mode(0o700)) {
                Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                Err(_) => return Err(unavailable("cannot create state directory")),
            }
            rustix::fs::openat(directory, name, flags, Mode::empty())
                .map_err(|_| unavailable("state directory changed"))?
        }
        Err(_) => return Err(unavailable("cannot open ordinary state directory")),
    };
    Ok(Some(child.into()))
}
pub(super) fn anchor(directory: &File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()))
}
fn flags() -> i32 {
    i32::try_from((OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC).bits())
        .expect("native nofollow, nonblocking and cloexec flags fit i32")
}
pub(super) struct Lock(File);
impl Lock {
    pub(super) fn check(&self, path: &Path) -> Result<(), RpcError> {
        let opened = self
            .0
            .metadata()
            .map_err(|_| unavailable("cannot inspect package-state lock"))?;
        let current = fs::symlink_metadata(path)
            .map_err(|_| unavailable("package-state lock disappeared"))?;
        if !current.is_file() || opened.dev() != current.dev() || opened.ino() != current.ino() {
            return Err(unavailable("package-state lock identity changed"));
        }
        Ok(())
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
pub(super) fn lock(path: &Path, create: bool) -> Result<Option<Lock>, RpcError> {
    let Some(file) = open_lock(path, create)? else {
        return Ok(None);
    };
    check_lock(&file, path)?;
    let acquired = if create {
        file.try_lock()
    } else {
        file.try_lock_shared()
    };
    match acquired {
        Ok(()) => Ok(Some(Lock(file))),
        Err(std::fs::TryLockError::WouldBlock) => {
            Err(error(ErrorKind::Busy, "Package-state lock is busy"))
        }
        Err(_) => Err(unavailable("cannot acquire package-state lock")),
    }
}
fn open_lock(path: &Path, create: bool) -> Result<Option<File>, RpcError> {
    let file = match OpenOptions::new()
        .read(true)
        .write(create)
        .create(create)
        .truncate(false)
        .mode(0o600)
        .custom_flags(flags())
        .open(path)
    {
        Ok(file) => file,
        Err(e)
            if !create
                && e.kind() == std::io::ErrorKind::NotFound
                && fs::symlink_metadata(path).is_err() =>
        {
            return Ok(None);
        }
        Err(_) => {
            return Err(unavailable(
                "Cannot open ordinary private package-state lock",
            ));
        }
    };
    Ok(Some(file))
}
fn check_lock(file: &File, path: &Path) -> Result<(), RpcError> {
    let opened = file
        .metadata()
        .map_err(|_| unavailable("cannot inspect package-state lock"))?;
    let current =
        fs::symlink_metadata(path).map_err(|_| unavailable("package-state lock disappeared"))?;
    if opened.mode() & 0o077 != 0
        || !opened.is_file()
        || !current.is_file()
        || opened.dev() != current.dev()
        || opened.ino() != current.ino()
    {
        return Err(unavailable("package-state lock identity changed"));
    }
    Ok(())
}
pub(super) fn read(path: &Path) -> Result<Option<Vec<u8>>, RpcError> {
    let mut file = match OpenOptions::new()
        .read(true)
        .custom_flags(flags())
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(unavailable("cannot read ordinary package store")),
    };
    let before = store_metadata(&file)?;
    let mut bytes =
        Vec::with_capacity(usize::try_from(before.len()).expect("store length is at most 4 MiB"));
    Read::by_ref(&mut file)
        .take(codec::BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable("cannot finish reading package store"))?;
    if bytes.len() > codec::BYTES {
        return Err(limit("encoded package store"));
    }
    check_read(&file, path, &before)?;
    Ok(Some(bytes))
}

fn store_metadata(file: &File) -> Result<fs::Metadata, RpcError> {
    let before = file
        .metadata()
        .map_err(|_| unavailable("cannot inspect package store"))?;
    if before.mode() & 0o077 != 0 || !before.is_file() {
        return Err(invalid("package store file type"));
    }
    if before.len() > codec::BYTES as u64 {
        return Err(limit("encoded package store"));
    }
    Ok(before)
}
fn check_read(file: &File, path: &Path, before: &fs::Metadata) -> Result<(), RpcError> {
    let after = file
        .metadata()
        .map_err(|_| unavailable("cannot finish inspecting package store"))?;
    let current =
        fs::symlink_metadata(path).map_err(|_| unavailable("package store disappeared"))?;
    if before.len() != after.len()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
        || !current.is_file()
        || current.dev() != after.dev()
        || current.ino() != after.ino()
    {
        return Err(conflict("package store changed during read"));
    }
    Ok(())
}

pub(super) fn private(directory: &File) -> Result<(), RpcError> {
    if directory
        .metadata()
        .map_err(|_| unavailable("Cannot inspect package directory"))?
        .mode()
        & 0o077
        != 0
    {
        return Err(invalid("Package state directory must be private (0700)"));
    }
    Ok(())
}
pub(super) fn same_file(file: &File, path: &Path) -> Result<(), RpcError> {
    let opened = file
        .metadata()
        .map_err(|_| unavailable("Cannot inspect staged package state"))?;
    let current =
        fs::symlink_metadata(path).map_err(|_| unavailable("Staged package state disappeared"))?;
    if !current.is_file() || opened.dev() != current.dev() || opened.ino() != current.ino() {
        return Err(unavailable("Staged package state identity changed"));
    }
    Ok(())
}

/// Native staging owns its file, not an externally substituted pathname.
/// Preserve an unknown entry on every early return, including IO faults.
pub(super) struct Staging(crate::file::temporary::Temporary);
impl Staging {
    pub fn new(parent: &Path) -> Result<Self, RpcError> {
        crate::file::temporary::Temporary::new(parent)
            .map(Self)
            .map_err(|_| unavailable("Cannot stage package state"))
    }
    pub fn file(&mut self) -> &mut File {
        &mut self.0.file
    }
    pub fn path(&self) -> &Path {
        &self.0.path
    }
    pub fn check(&self) -> Result<(), RpcError> {
        same_file(&self.0.file, &self.0.path)
    }
    pub fn disarm(&mut self) {
        self.0.disarm();
    }
}
impl Drop for Staging {
    fn drop(&mut self) {
        if self.check().is_err() {
            self.0.disarm();
        }
    }
}
