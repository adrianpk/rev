//! Session-owned named file locations. Source text and view selections are never
//! owned or mutated by this collection.

pub(crate) mod context;
pub(crate) mod list;
pub(crate) mod store;
pub use context::Context;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod lifecycle_tests;

/// Borrowed collection observation. Mutation remains with native owners.
pub struct Collection<'a>(pub(crate) std::sync::MutexGuard<'a, Bookmarks>);
impl std::ops::Deref for Collection<'_> {
    type Target = Bookmarks;
    fn deref(&self) -> &Bookmarks {
        &self.0
    }
}

pub(crate) type Shared = Arc<Mutex<Bookmarks>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution {
    Resolved,
    Stale,
    Unresolved,
    Unavailable,
}

/// Text history retains only scalar outcomes and explicit target generations.
/// No source snapshots or context strings belong to these observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Anchor {
    pub id: RecordId,
    pub generation: u64,
    pub position: Option<crate::coordinates::Scalar>,
}

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
    generation: u64,
    resolution: Resolution,
    context: Context,
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
    /// A revision-qualified observation. Only resolved records have a current
    /// live boundary; stale and closed records retain their last point evidence.
    pub fn context(&self) -> &Context {
        &self.context
    }
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }
    pub fn point(&self) -> Point {
        self.point
    }
}

pub struct Bookmarks {
    entries: BTreeMap<String, Record>,
    next_id: u64,
    dirty: bool,
    generation: u64,
    pub(crate) max_entries: usize,
    pub(crate) context_chars: usize,
}

impl Default for Bookmarks {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            next_id: 0,
            dirty: false,
            generation: 0,
            max_entries: MAX_ENTRIES,
            context_chars: context::CONTEXT_CHARS,
        }
    }
}

impl Bookmarks {
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
    fn changed(&mut self) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("bookmark collection generation exhausted");
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub(crate) fn saved(&mut self) {
        self.dirty = false;
    }
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
        context: Context,
    ) -> Result<RecordId, String> {
        if name.is_empty() || name.len() > MAX_NAME_BYTES || name.chars().any(char::is_control) {
            return Err("Bookmark names must contain 1..256 UTF-8 bytes without controls".into());
        }
        let id = match self.entries.get(name) {
            Some(_) if !overwrite => return Err("Bookmark name already exists".into()),
            Some(record) => record.id,
            None => {
                if self.entries.len() >= self.max_entries {
                    return Err(format!(
                        "Bookmark collection is full ({} entries)",
                        self.max_entries
                    ));
                }
                RecordId(
                    self.next_id
                        .checked_add(1)
                        .ok_or("Bookmark identity exhausted")?,
                )
            }
        };
        let generation = self
            .entries
            .get(name)
            .map_or(Some(1), |record| record.generation.checked_add(1))
            .ok_or("Bookmark generation exhausted")?;
        let record = Record {
            id,
            name: name.into(),
            path,
            point,
            generation,
            resolution: Resolution::Resolved,
            context,
        };
        self.entries.insert(name.into(), record);
        self.next_id = self.next_id.max(id.0);
        self.dirty = true;
        self.changed();
        Ok(id)
    }

    pub(crate) fn delete(&mut self, name: &str) -> Result<(), String> {
        self.entries
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| "No bookmark with that name".to_owned())?;
        self.dirty = true;
        self.changed();
        Ok(())
    }
}

impl Bookmarks {
    pub(crate) fn anchors(
        &self,
        stamp: crate::buffer::Stamp,
    ) -> Result<Vec<Anchor>, crate::buffer::edit::Error> {
        let mut anchors = Vec::new();
        for record in self.entries.values().filter(|r| {
            r.point.stamp.buffer == stamp.buffer
                && matches!(r.resolution, Resolution::Resolved | Resolution::Stale)
        }) {
            if record.point.stamp != stamp {
                return Err(crate::buffer::edit::Error::ChangedPositions);
            }
            anchors.push(Anchor {
                id: record.id,
                generation: record.generation,
                position: (record.resolution == Resolution::Resolved)
                    .then_some(record.point.position),
            });
        }
        anchors.sort_by_key(|a| a.id);
        Ok(anchors)
    }
    pub(crate) fn publish_anchors(&mut self, stamp: crate::buffer::Stamp, anchors: &[Anchor]) {
        if !anchors.is_empty() {
            self.dirty = true;
            self.changed();
        }
        for record in self.entries.values_mut() {
            let Ok(index) = anchors.binary_search_by_key(&record.id, |a| a.id) else {
                continue;
            };
            let anchor = anchors[index];
            if anchor.generation != record.generation {
                continue;
            }
            record.point.stamp = stamp;
            record.resolution = if let Some(position) = anchor.position {
                record.point.position = position;
                Resolution::Resolved
            } else {
                Resolution::Stale
            };
        }
    }
}

impl Bookmarks {
    pub(crate) fn checkpoint(
        &mut self,
        buffer: &Buffer,
        closing: bool,
    ) -> Result<(), crate::buffer::Error> {
        // Capture all fallible outcomes before changing any record.
        let captured = self
            .entries
            .values()
            .filter(|r| r.point.stamp.buffer == buffer.id() && r.resolution == Resolution::Resolved)
            .map(|r| {
                Context::capture_with(buffer, r.point.position, self.context_chars)
                    .map(|context| (r.id, context))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let mut changed = false;
        for record in self.entries.values_mut().filter(|r| {
            r.point.stamp.buffer == buffer.id()
                && matches!(r.resolution, Resolution::Resolved | Resolution::Stale)
        }) {
            if let Some(context) = captured.get(&record.id) {
                changed |= record.context != *context;
                self.dirty |= record.context != *context;
                record.context = context.clone();
            }
            if closing {
                changed = true;
                record.resolution = Resolution::Unresolved;
            }
        }
        if changed {
            self.changed();
        }
        Ok(())
    }
    pub(crate) fn resolved(
        &mut self,
        id: RecordId,
        generation: u64,
        point: Point,
        context: Context,
        path: PathBuf,
    ) -> Result<(), String> {
        let record = self
            .entries
            .values_mut()
            .find(|r| r.id == id && r.generation == generation)
            .ok_or("Bookmark changed during navigation")?;
        let changed = record.point != point
            || record.context != context
            || record.path != path
            || record.resolution != Resolution::Resolved;
        self.dirty |= record.context != context || record.path != path;
        record.point = point;
        record.path = path;
        record.context = context;
        record.resolution = Resolution::Resolved;
        if changed {
            self.changed();
        }
        Ok(())
    }
    pub(crate) fn unavailable(&mut self, id: RecordId, generation: u64) {
        if let Some(record) = self.entries.values_mut().find(|r| {
            r.id == id
                && r.generation == generation
                && !matches!(r.resolution, Resolution::Resolved | Resolution::Stale)
        }) {
            if record.resolution != Resolution::Unavailable {
                record.resolution = Resolution::Unavailable;
                self.changed();
            }
        }
    }
    pub(crate) fn retarget(
        &mut self,
        buffer: crate::buffer::BufferId,
        prior: Option<&Path>,
        target: &Path,
    ) {
        let mut changed = false;
        for record in self.entries.values_mut().filter(|r| {
            (r.point.stamp.buffer == buffer
                && matches!(r.resolution, Resolution::Resolved | Resolution::Stale))
                || prior == Some(r.path.as_path())
        }) {
            changed |= record.path != target;
            self.dirty |= record.path != target;
            record.path = target.into();
        }
        if changed {
            self.changed();
        }
    }
    pub(crate) fn renamed_paths(
        &self,
        source: &Path,
        target: &Path,
        directory: bool,
    ) -> Result<Vec<(RecordId, PathBuf)>, String> {
        let mut paths = Vec::new();
        for record in self.entries.values() {
            let Ok(suffix) = record.path.strip_prefix(source) else {
                continue;
            };
            if !directory && !suffix.as_os_str().is_empty() {
                continue;
            }
            let path = if suffix.as_os_str().is_empty() {
                target.to_owned()
            } else {
                target.join(suffix)
            };
            if path.as_os_str().as_encoded_bytes().len() > MAX_PATH_BYTES {
                return Err("Renamed bookmark path exceeds 4096 bytes".into());
            }
            paths.push((record.id, path));
        }
        Ok(paths)
    }
    pub(crate) fn publish_paths(&mut self, paths: Vec<(RecordId, PathBuf)>) {
        let mut changed = false;
        for (id, path) in paths {
            if let Some(record) = self.entries.values_mut().find(|r| r.id == id) {
                changed |= record.path != path;
                self.dirty |= record.path != path;
                record.path = path;
            }
        }
        if changed {
            self.changed();
        }
    }
}
