use super::{Anchor, Source};
use crate::buffer::{
    Buffer, Stamp,
    edit::{Error, Point},
};
use crate::coordinates::{Coordinates, Scalar};
use crate::file::Context;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak},
};

pub(crate) type Shared = Arc<Mutex<Registry>>;
pub(crate) type Closing = Vec<(Arc<Mutex<Source>>, Source)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Snapshot {
    pub id: u64,
    pub position: Option<Scalar>,
}

#[derive(Default)]
pub(crate) struct Registry {
    next: u64,
    entries: BTreeMap<u64, Weak<Mutex<Source>>>,
}

impl Registry {
    pub(crate) fn capture(&mut self, buffer: &Buffer, point: Point) -> Result<Anchor, Error> {
        buffer.slice(&crate::buffer::Span {
            stamp: point.stamp,
            range: point.position.0..point.position.0,
        })?;
        if !Coordinates::new(buffer)?.is_boundary(point.position)? {
            return Err(crate::coordinates::Error::Cluster(point.position).into());
        }
        let context = Context::capture(buffer, point.position)?;
        let file = buffer.file().cloned();
        if file
            .as_ref()
            .is_some_and(|f| f.path().as_os_str().as_encoded_bytes().len() > 4096)
        {
            return Err(Error::Capacity);
        }
        let id = self.next.checked_add(1).ok_or(Error::Capacity)?;
        let source = Arc::new(Mutex::new(Source {
            point,
            position: Some(point.position),
            file,
            context,
            closed: false,
        }));
        self.prune();
        self.entries.insert(id, Arc::downgrade(&source));
        self.next = id;
        Ok(Anchor { source })
    }

    fn prune(&mut self) {
        self.entries.retain(|_, source| source.strong_count() > 0);
    }

    pub(crate) fn anchors(&mut self, stamp: Stamp) -> Result<Vec<Snapshot>, Error> {
        self.prune();
        let mut anchors = Vec::new();
        for (&id, source) in &self.entries {
            let Some(source) = source.upgrade() else {
                continue;
            };
            let source = source.lock().expect("location owner poisoned");
            if source.closed || source.point.stamp.buffer != stamp.buffer {
                continue;
            }
            if source.point.stamp != stamp {
                return Err(Error::ChangedPositions);
            }
            anchors.push(Snapshot {
                id,
                position: source.position,
            });
        }
        Ok(anchors)
    }

    pub(crate) fn publish(&mut self, stamp: Stamp, anchors: &[Snapshot]) {
        for anchor in anchors {
            if let Some(source) = self.entries.get(&anchor.id).and_then(Weak::upgrade) {
                let mut source = source.lock().expect("location owner poisoned");
                source.point.stamp = stamp;
                source.position = anchor.position;
                if let Some(point) = anchor.position {
                    source.point.position = point;
                }
            }
        }
    }

    /// Prepare every fallible source snapshot before publishing close metadata.
    pub(crate) fn prepare_close(&self, buffer: &Buffer) -> Result<Closing, crate::buffer::Error> {
        let mut candidates = Vec::new();
        for source in self.entries.values().filter_map(Weak::upgrade) {
            let mut current = source.lock().expect("location owner poisoned").clone();
            if current.point.stamp.buffer != buffer.id() || current.closed {
                continue;
            }
            if let Some(point) = current.position {
                current.context = Context::capture(buffer, point)?;
            }
            current.file = buffer.file().cloned();
            current.closed = true;
            candidates.push((source, current));
        }
        Ok(candidates)
    }

    pub(crate) fn publish_close(&self, candidates: Closing) {
        for (source, current) in candidates {
            *source.lock().expect("location owner poisoned") = current;
        }
    }

    /// Reopening validates each closed observation independently. A location
    /// whose evidence cannot be fully resolved remains closed, never guessed.
    pub(crate) fn reopened(&mut self, buffer: &Buffer) {
        self.prune();
        for source in self.entries.values().filter_map(Weak::upgrade) {
            let mut source = source.lock().expect("location owner poisoned");
            if reopens(&source, buffer) {
                source.point.stamp = buffer.stamp();
                source.closed = false;
            }
        }
    }

    pub(crate) fn renamed(
        &mut self,
        from: &std::path::Path,
        to: &std::path::Path,
        directory: bool,
    ) {
        self.prune();
        for source in self.entries.values().filter_map(Weak::upgrade) {
            let mut source = source.lock().expect("location owner poisoned");
            let Some(file) = source.file.as_ref() else {
                continue;
            };
            let Ok(suffix) = file.path().strip_prefix(from) else {
                continue;
            };
            if !directory && !suffix.as_os_str().is_empty() {
                continue;
            }
            source.file = Some(file.relocated(to.join(suffix)));
        }
    }
}

fn reopens(source: &Source, buffer: &Buffer) -> bool {
    if !source.closed || buffer.is_modified() {
        return false;
    }
    let Some(position) = source.position else {
        return false;
    };
    let (Some(prior), Some(current)) = (source.file.as_ref(), buffer.file()) else {
        return false;
    };
    if prior.path() != current.path() {
        return false;
    }
    let identity = prior
        .observation()
        .zip(current.observation())
        .is_some_and(|(before, after)| before.same_content(after));
    identity && context_agrees(source, buffer, position)
}

fn context_agrees(source: &Source, buffer: &Buffer, position: Scalar) -> bool {
    source.context.matches(buffer, position.0) == Ok(true)
        && Coordinates::new(buffer)
            .ok()
            .and_then(|coordinates| coordinates.is_boundary(position).ok())
            == Some(true)
}
