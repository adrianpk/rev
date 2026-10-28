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
