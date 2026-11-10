mod format;

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use crate::file::{Files, Output, temporary::Temporary};

const BYTES: usize = 64 * 1024;

/// Bounded storage failures contain no file contents or command arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Busy,
    Invalid,
    Unsupported,
    Bound,
    Unavailable(&'static str),
    Ambiguous,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Busy => "publication lock is busy; usage remains pending",
            Self::Invalid => "invalid history; file preserved and persistence suspended",
            Self::Unsupported => {
                "unsupported history version; file preserved and persistence suspended"
            }
            Self::Bound => "history exceeds its byte, name or event bound; file preserved",
            Self::Unavailable(operation) => operation,
            Self::Ambiguous => {
                "publication completion is unresolved; persistence suspended to avoid duplicate usage"
            }
        };
        write!(
            f,
            "Command usage: {message}; session ranking remains available"
        )
    }
}
impl std::error::Error for Error {}

impl Error {
    pub(super) fn suspends(self) -> bool {
        matches!(
            self,
            Self::Invalid | Self::Unsupported | Self::Bound | Self::Ambiguous
        )
    }
}

/// One cooperating publisher. Open reads without creating files. Publication
/// rebases only the supplied pending events onto the latest valid disk window.
pub struct Store {
    path: PathBuf,
    suspended: bool,
}

impl Store {
    /// Read a bounded absolute path and return its oldest-first event window.
    /// A missing file is empty; invalid existing entries are never rewritten.
    pub fn open(path: &Path) -> Result<(Self, Vec<String>), Error> {
        if !path.is_absolute() {
            return Err(Error::Unavailable("history requires an absolute path"));
        }
        let bytes = read(path)?;
        let events = format::decode(bytes.as_deref())?;
        Ok((
            Self {
                path: path.into(),
                suspended: false,
            },
            events,
        ))
    }

    /// Append one caller-owned pending batch and return the rebased window.
    /// Discard an acknowledged batch after success. Ambiguous completion
    /// suspends this store so retrying that batch cannot duplicate its events.
    pub fn publish(&mut self, pending: &[String]) -> Result<Vec<String>, Error> {
        self.publish_with(pending, &mut Files)
    }

    /// Uses the ordinary concrete file-publication boundary. A failed replace
    /// means no replacement occurred; a later fault suspends this publisher.
    pub fn publish_with(
        &mut self,
        pending: &[String],
        output: &mut impl Output,
    ) -> Result<Vec<String>, Error> {
        if self.suspended {
            return Err(Error::Ambiguous);
        }
        format::validate(pending)?;
        if pending.is_empty() {
            return format::decode(read(&self.path)?.as_deref());
        }
        let parent = self.path.parent().ok_or(Error::Invalid)?;
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)
            .map_err(|_| Error::Unavailable("cannot create the state directory"))?;
        let _lock = lock(&self.path)?;
        let before = read(&self.path)?;
        let mut events = format::decode(before.as_deref())?;
        let discard = (events.len() + pending.len()).saturating_sub(super::WINDOW);
        events.drain(..discard);
        events.extend_from_slice(pending);
        let bytes = format::encode(&events)?;
        if before.is_some() {
            OpenOptions::new()
                .write(true)
                .open(&self.path)
                .map_err(|_| Error::Unavailable("history is not writable"))?;
        }
        self.replace(&bytes, before.as_deref(), output)?;
        Ok(events)
    }

    fn replace(
        &mut self,
        bytes: &[u8],
        before: Option<&[u8]>,
        output: &mut impl Output,
    ) -> Result<(), Error> {
        let parent = self.path.parent().ok_or(Error::Invalid)?;
        let directory = File::open(parent)
            .map_err(|_| Error::Unavailable("cannot open the state directory"))?;
        let mut temporary = Temporary::new(parent)
            .map_err(|_| Error::Unavailable("cannot stage command history"))?;
        temporary
            .file
            .write_all(bytes)
            .and_then(|()| output.flush(&mut temporary.file))
            .and_then(|()| output.sync_file(&temporary.file))
            .map_err(|_| Error::Unavailable("cannot synchronize staged command history"))?;
        if read(&self.path)?.as_deref() != before {
            return Err(Error::Unavailable(
                "history changed during publication; pending usage retained",
            ));
        }
        output
            .replace(&temporary.path, &self.path)
            .map_err(|_| Error::Unavailable("cannot replace command history"))?;
        temporary.disarm();
        // After replacement, even an observation failure cannot justify
        // appending this batch again. Suspend rather than guessing its outcome.
        let completed = output
            .sync_directory(&directory)
            .and_then(|()| output.observe(&self.path).map(|_| ()))
            .is_ok()
            && read(&self.path).is_ok_and(|actual| actual.as_deref() == Some(bytes));
        if !completed {
            self.suspended = true;
            return Err(Error::Ambiguous);
        }
        Ok(())
    }
}

struct Lock(File);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

fn flags() -> i32 {
    i32::try_from((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits())
        .expect("file flags")
}

fn lock(path: &Path) -> Result<Lock, Error> {
    let mut name = path.as_os_str().to_owned();
    name.push(".lock");
    let path = PathBuf::from(name);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(flags())
        .open(&path)
        .map_err(|_| Error::Unavailable("cannot open the publication lock"))?;
    let opened = file
        .metadata()
        .map_err(|_| Error::Unavailable("cannot inspect the publication lock"))?;
    let current = fs::symlink_metadata(path)
        .map_err(|_| Error::Unavailable("publication lock disappeared"))?;
    if !opened.is_file()
        || !current.is_file()
        || opened.dev() != current.dev()
        || opened.ino() != current.ino()
    {
        return Err(Error::Unavailable("publication lock changed"));
    }
    match file.try_lock() {
        Ok(()) => Ok(Lock(file)),
        Err(std::fs::TryLockError::WouldBlock) => Err(Error::Busy),
        Err(_) => Err(Error::Unavailable("cannot acquire the publication lock")),
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
        Err(_) => return Err(Error::Unavailable("cannot read command history")),
    };
    let before = file
        .metadata()
        .map_err(|_| Error::Unavailable("cannot inspect command history"))?;
    if !before.is_file() {
        return Err(Error::Invalid);
    }
    if before.len() > BYTES as u64 {
        return Err(Error::Bound);
    }
    let mut bytes = vec![0; usize::try_from(before.len()).expect("bounded command history size")];
    file.read_exact(&mut bytes)
        .map_err(|_| Error::Unavailable("history changed during reading"))?;
    let mut extra = [0];
    if file
        .read(&mut extra)
        .map_err(|_| Error::Unavailable("cannot finish reading command history"))?
        != 0
    {
        return Err(Error::Bound);
    }
    let after = file
        .metadata()
        .map_err(|_| Error::Unavailable("cannot finish inspecting command history"))?;
    let current = fs::symlink_metadata(path)
        .map_err(|_| Error::Unavailable("history changed during reading"))?;
    if before.len() != after.len()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
        || !current.is_file()
        || current.dev() != after.dev()
        || current.ino() != after.ino()
    {
        return Err(Error::Unavailable("history changed during reading"));
    }
    Ok(Some(bytes))
}
