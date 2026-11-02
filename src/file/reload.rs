//! Bounded disk ingress for visiting-buffer refresh. Call only on an IO worker.
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::{Duration, Instant};

use rustix::fs::{Mode, OFlags, open};

use super::{Loaded, Location, Observation, observe};

pub(crate) const DEADLINE: Duration = Duration::from_secs(5);

pub(crate) struct Image {
    pub loaded: Loaded,
    pub observed: Observation,
}

pub(crate) fn read(location: &Location, max_bytes: usize) -> io::Result<Image> {
    let started = Instant::now();
    for attempt in 0..=2 {
        match read_once(location.path(), max_bytes, started) {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock && attempt < 2 => {}
            result => return result,
        }
    }
    unreachable!("bounded attempts return their last result")
}

fn read_once(path: &Path, max_bytes: usize, started: Instant) -> io::Result<Image> {
    association(path)?;
    let observed = present(path)?;
    if observed.length > max_bytes as u64 {
        return Err(io::Error::other(
            "reload source exceeds configured byte limit",
        ));
    }
    // NONBLOCK prevents a regular-file-to-FIFO substitution from pinning a read.
    // NOFOLLOW rejects a newly substituted final symlink before opening it.
    let descriptor = open(
        path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
        Mode::empty(),
    )?;
    let file = File::from(descriptor);
    if Observation::from_metadata(file.metadata()?)? != observed {
        return Err(unstable());
    }
    let mut reader = Limited {
        source: file.take(max_bytes as u64 + 1),
        started,
    };
    let loaded = Loaded::read(&mut reader)?;
    if reader.source.limit() == 0 {
        return Err(io::Error::other(
            "reload source exceeds configured byte limit",
        ));
    }
    if max_bytes as u64 + 1 - reader.source.limit() != observed.length
        || Observation::from_metadata(reader.source.get_ref().metadata()?)? != observed
    {
        return Err(unstable());
    }
    check(path, &observed)?;
    deadline(started)?;
    Ok(Image { loaded, observed })
}

pub(crate) fn check(path: &Path, expected: &Observation) -> io::Result<()> {
    association(path)?;
    if present(path)? != *expected {
        return Err(unstable());
    }
    Ok(())
}

fn association(path: &Path) -> io::Result<()> {
    if std::fs::canonicalize(path)? != path {
        return Err(io::Error::other("reload visiting association changed"));
    }
    Ok(())
}

fn present(path: &Path) -> io::Result<Observation> {
    observe(path)?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "reload source is missing"))
}

fn unstable() -> io::Error {
    io::Error::new(
        io::ErrorKind::WouldBlock,
        "reload source changed; invoke revert-buffer again",
    )
}

pub(crate) fn deadline(started: Instant) -> io::Result<()> {
    if started.elapsed() > DEADLINE {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "reload admission exceeded five seconds",
        ));
    }
    Ok(())
}

struct Limited {
    source: std::io::Take<File>,
    started: Instant,
}

impl Read for Limited {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        deadline(self.started)?;
        self.source.read(bytes)
    }
}
