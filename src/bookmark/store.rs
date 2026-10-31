//! Bounded, observed user data with a nonblocking cooperating-writer lock.
//! All staging faults use the existing concrete file Output boundary.
mod format;
#[cfg(test)]
mod tests;

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use super::{Bookmarks, Record, RecordId, Resolution};
use crate::file::{Files, Observation, Output, observe, temporary::Temporary};
pub(crate) use format::Stored;

const LIMIT: usize = 16 * 1024 * 1024;

#[derive(PartialEq, Eq)]
struct Evidence {
    observation: Option<Observation>,
    bytes: Vec<u8>,
}

pub(crate) struct Store {
    path: PathBuf,
    expected: Evidence,
    // A successful replacement followed by a failed sync/observation remains
    // unresolved. Only matching actual bytes can reconcile that publication.
    pending: Option<Vec<u8>>,
}
impl Store {
    pub(crate) fn open(path: &Path, limit: usize) -> Result<(Self, Vec<Stored>), String> {
        if !path.is_absolute() {
            return Err("Bookmark store requires an absolute path".into());
        }
        let expected = read(path)?;
        let records = if expected.observation.is_some() {
            format::decode(&expected.bytes, limit)?
        } else {
            Vec::new()
        };
        Ok((
            Self {
                path: path.into(),
                expected,
                pending: None,
            },
            records,
        ))
    }
    pub(crate) fn write(&mut self, bookmarks: &Bookmarks) -> Result<(), String> {
        self.write_with(bookmarks, &mut Files)
    }
    pub(crate) fn write_with(
        &mut self,
        bookmarks: &Bookmarks,
        output: &mut impl Output,
    ) -> Result<(), String> {
        let bytes = format::encode(bookmarks)?;
        let parent = self
            .path
            .parent()
            .ok_or("Bookmark store has no parent")?
            .to_owned();
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&parent)
            .map_err(|e| e.to_string())?;
        let _lock = self.lock()?;
        if let Some(pending) = self.pending.take() {
            if let Err(error) = self.complete(&pending, output) {
                self.pending = Some(pending);
                return Err(error);
            }
            if pending == bytes {
                return Ok(());
            }
        }
        self.guard()?;
        let directory = File::open(&parent).map_err(|e| e.to_string())?;
        let mut temporary = Temporary::new(&parent).map_err(|e| e.to_string())?;
        let mode = self
            .expected
            .observation
            .as_ref()
            .map_or(0o600, |file| file.mode & 0o777);
        let staged = temporary
            .file
            .set_permissions(fs::Permissions::from_mode(mode))
            .and_then(|()| temporary.file.write_all(&bytes))
            .and_then(|()| output.flush(&mut temporary.file))
            .and_then(|()| output.sync_file(&temporary.file));
        if let Err(error) = staged {
            return Err(temporary.failed(error, output).to_string());
        }
        let replaced = self
            .guard()
            .map_err(std::io::Error::other)
            .and_then(|()| output.replace(&temporary.path, &self.path));
        if let Err(error) = replaced {
            return Err(temporary.failed(error, output).to_string());
        }
        temporary.disarm();
        // Retain the opened directory across publication, as the source owner
        // does, and verify both file identity and exact content before admission.
        let synced = output.sync_directory(&directory);
        if let Err(error) = synced.and_then(|()| self.verify(&bytes, output)) {
            self.pending = Some(bytes);
            return Err(format!(
                "Bookmark store published; completion unresolved: {error}"
            ));
        }
        Ok(())
    }
    fn lock(&self) -> Result<File, String> {
        let mut name = self.path.as_os_str().to_owned();
        name.push(".lock");
        let path = PathBuf::from(name);
        // Never follow an existing symlink or accept a changed lock inode.
        let before = observe(&path).map_err(|e| e.to_string())?;
        let file = if before.is_none() {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
        } else {
            OpenOptions::new().read(true).write(true).open(&path)
        }
        .map_err(|e| format!("Bookmark lock: {e}"))?;
        let actual = observe(&path)
            .map_err(|e| e.to_string())?
            .ok_or("Bookmark lock disappeared")?;
        use std::os::unix::fs::MetadataExt;
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        if actual.device != metadata.dev()
            || actual.inode != metadata.ino()
            || before
                .as_ref()
                .is_some_and(|b| b.device != actual.device || b.inode != actual.inode)
        {
            return Err("Bookmark lock changed".into());
        }
        file.try_lock()
            .map_err(|e| format!("Bookmark store is busy; retry: {e}"))?;
        Ok(file)
    }
    pub(crate) fn guard(&self) -> Result<(), String> {
        if read(&self.path)? != self.expected {
            return Err(
                "Bookmark store changed externally; memory and disk retained; reload before saving"
                    .into(),
            );
        }
        if self.expected.observation.is_some() {
            // Match ordinary source publication: directory write permission is
            // not permission to bypass a read-only target by replacing its inode.
            OpenOptions::new()
                .write(true)
                .open(&self.path)
                .map_err(|e| format!("Bookmark store is not writable: {e}"))?;
        }
        Ok(())
    }
    fn complete(&mut self, bytes: &[u8], output: &mut impl Output) -> Result<(), String> {
        let current = read(&self.path)?;
        if current.observation.is_none() || current.bytes != bytes {
            return Err(
                "Unresolved bookmark publication conflicts with disk; memory and disk retained"
                    .into(),
            );
        }
        let parent = self.path.parent().ok_or("Bookmark store has no parent")?;
        let directory = File::open(parent).map_err(|e| e.to_string())?;
        output
            .sync_directory(&directory)
            .and_then(|()| self.verify(bytes, output))
            .map_err(|e| format!("Bookmark store published; completion unresolved: {e}"))
    }
    fn verify(&mut self, bytes: &[u8], output: &mut impl Output) -> std::io::Result<()> {
        let observation = output.observe(&self.path)?;
        let current = read(&self.path).map_err(std::io::Error::other)?;
        if current.observation.as_ref() != Some(&observation) || current.bytes != bytes {
            return Err(std::io::Error::other(
                "Published bookmark store changed before verification",
            ));
        }
        self.expected = current;
        Ok(())
    }
}

fn read(path: &Path) -> Result<Evidence, String> {
    let observation = observe(path).map_err(|e| format!("Bookmark store: {e}"))?;
    let mut bytes = Vec::new();
    if let Some(expected) = &observation {
        let len = usize::try_from(expected.length).map_err(|_| "Bookmark store exceeds 16 MiB")?;
        if len > LIMIT {
            return Err("Bookmark store exceeds 16 MiB".into());
        }
        // Allocate exactly the observed bounded length. Probe growth with one
        // stack byte, never an allocation beyond the hard store bound.
        bytes.resize(len, 0);
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        file.read_exact(&mut bytes).map_err(|e| e.to_string())?;
        let mut extra = [0];
        if file.read(&mut extra).map_err(|e| e.to_string())? != 0 {
            return Err("Bookmark store changed during reading".into());
        }
    }
    if observe(path).map_err(|e| e.to_string())? != observation {
        return Err("Bookmark store changed during reading".into());
    }
    Ok(Evidence { observation, bytes })
}

impl Bookmarks {
    pub(crate) fn admit(&mut self, records: Vec<Stored>) -> Result<(), String> {
        let next = self
            .next_id
            .checked_add(records.len() as u64)
            .ok_or("Bookmark identity exhausted")?;
        let mut entries = std::collections::BTreeMap::new();
        for (offset, stored) in records.into_iter().enumerate() {
            let id = RecordId(self.next_id + offset as u64 + 1);
            let point = crate::buffer::edit::Point {
                stamp: crate::buffer::Stamp {
                    buffer: crate::buffer::BufferId::UNBOUND,
                    revision: 0,
                },
                position: crate::coordinates::Scalar(stored.context.position),
            };
            let record = Record {
                id,
                name: stored.name.clone(),
                path: stored.path,
                point,
                generation: 1,
                resolution: Resolution::Unresolved,
                context: stored.context,
            };
            entries.insert(stored.name, record);
        }
        self.entries = entries;
        self.next_id = next;
        self.saved();
        self.changed();
        Ok(())
    }
}
