//! Session-owned named file locations. Source text and view selections are never
//! owned or mutated by this collection.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::buffer::{Buffer, edit::Point};

pub const MAX_ENTRIES: usize = 1024;
pub const MAX_NAME_BYTES: usize = 256;
pub const MAX_PATH_BYTES: usize = 4096;

/// Stable within a session; replacing a name retains its identity, while
/// deleting and recreating it allocates a fresh identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordId(u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    id: RecordId,
    name: String,
    path: PathBuf,
    point: Point,
}

impl Record {
    pub fn id(&self) -> RecordId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// A revision-qualified location; callers must check it before navigation.
    pub fn point(&self) -> Point {
        self.point
    }
}

#[derive(Default)]
pub struct Bookmarks {
    entries: BTreeMap<String, Record>,
    next_id: u64,
}

impl Bookmarks {
    pub fn get(&self, name: &str) -> Option<&Record> {
        self.entries.get(name)
    }
    pub fn iter(&self) -> impl Iterator<Item = &Record> {
        self.entries.values()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn target(buffer: &Buffer, point: Point) -> Result<PathBuf, String> {
        if buffer.is_transient() {
            return Err("Bookmarks require a file-backed source".into());
        }
        let file = buffer
            .file()
            .ok_or("Bookmarks require a file-backed source")?;
        if file.path().as_os_str().as_encoded_bytes().len() > MAX_PATH_BYTES {
            return Err("Bookmark path exceeds 4096 bytes".into());
        }
        if point.stamp != buffer.stamp() {
            return Err("Bookmark source changed".into());
        }
        buffer
            .span(point.position.0..point.position.0)
            .map_err(|e| e.to_string())?;
        Ok(file.path().to_owned())
    }

    pub(crate) fn set(
        &mut self,
        name: &str,
        path: PathBuf,
        point: Point,
        overwrite: bool,
    ) -> Result<RecordId, String> {
        if name.is_empty() || name.len() > MAX_NAME_BYTES || name.chars().any(char::is_control) {
            return Err("Bookmark names must contain 1..256 UTF-8 bytes without controls".into());
        }
        let id = match self.entries.get(name) {
            Some(_) if !overwrite => return Err("Bookmark name already exists".into()),
            Some(record) => record.id,
            None => {
                if self.entries.len() >= MAX_ENTRIES {
                    return Err("Bookmark collection is full (1024 entries)".into());
                }
                RecordId(
                    self.next_id
                        .checked_add(1)
                        .ok_or("Bookmark identity exhausted")?,
                )
            }
        };
        let record = Record {
            id,
            name: name.into(),
            path,
            point,
        };
        self.entries.insert(name.into(), record);
        self.next_id = self.next_id.max(id.0);
        Ok(id)
    }

    pub(crate) fn delete(&mut self, name: &str) -> Result<(), String> {
        self.entries
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| "No bookmark with that name".into())
    }
}
