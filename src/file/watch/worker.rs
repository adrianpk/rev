use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::fd::OwnedFd;
use rustix::fs::inotify::{self, ReadFlags, WatchFlags};

use super::{BATCH, Changed, Disk, QUEUE, Shared, Source, Wake};
use crate::buffer::BufferId;

struct Entry {
    source: Source,
    last: Disk,
    watched: bool,
}

struct Worker {
    descriptor: OwnedFd,
    shared: Arc<Mutex<Shared>>,
    wake: Arc<Wake>,
    entries: BTreeMap<BufferId, Entry>,
    parents: BTreeMap<PathBuf, i32>,
    due: BTreeMap<BufferId, Instant>,
    rescan: Option<usize>,
    rescan_again: bool,
    debounce: Duration,
}

pub(super) fn run(
    shared: Arc<Mutex<Shared>>,
    control: Arc<OwnedFd>,
    wake: Arc<Wake>,
    debounce_ms: usize,
) -> io::Result<()> {
    let mut worker = Worker {
        descriptor: inotify::init(inotify::CreateFlags::CLOEXEC | inotify::CreateFlags::NONBLOCK)?,
        shared,
        wake,
        entries: BTreeMap::new(),
        parents: BTreeMap::new(),
        due: BTreeMap::new(),
        rescan: None,
        rescan_again: false,
        debounce: Duration::from_millis(debounce_ms as u64),
    };
    loop {
        let update = {
            let mut state = worker.shared.lock().expect("file observation poisoned");
            if state.stop {
                return Ok(());
            }
            state.update.take()
        };
        if let Some(sources) = update {
            worker.update(sources);
        }
        worker.scan_batch();
        worker.check_batch();
        let timeout = worker.wait().map(|wait| Timespec {
            tv_sec: wait.as_secs() as _,
            tv_nsec: wait.subsec_nanos() as _,
        });
        let mut fds = [
            PollFd::new(&worker.descriptor, PollFlags::IN),
            PollFd::new(control.as_ref(), PollFlags::IN),
        ];
        match poll(&mut fds, timeout.as_ref()) {
            Err(rustix::io::Errno::INTR) => continue,
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
        let notified = fds[0].revents().contains(PollFlags::IN);
        if fds[1].revents().contains(PollFlags::IN) {
            let mut bytes = [0u8; 8];
            match rustix::io::read(control.as_ref(), &mut bytes) {
                Ok(_) | Err(rustix::io::Errno::AGAIN) => {}
                Err(error) => return Err(error.into()),
            }
        }
        if notified {
            worker.events()?;
        }
    }
}

impl Worker {
    fn update(&mut self, sources: Vec<Source>) {
        let wanted: BTreeSet<_> = sources.iter().map(|s| s.id).collect();
        self.entries.retain(|id, _| wanted.contains(id));
        self.due.retain(|id, _| wanted.contains(id));
        let parents: BTreeSet<_> = sources
            .iter()
            .filter_map(|s| s.path.parent().map(PathBuf::from))
            .collect();
        self.parents.retain(|path, wd| {
            if parents.contains(path) {
                return true;
            }
            match inotify::remove_watch(&self.descriptor, *wd) {
                Ok(()) | Err(rustix::io::Errno::INVAL) => {}
                Err(error) => {
                    self.shared
                        .lock()
                        .expect("file observation poisoned")
                        .diagnostic = Some(format!(
                        "retiring file observation: {error}; use revert-buffer"
                    ));
                    self.wake.notify();
                }
            }
            false
        });
        for source in sources {
            if self
                .entries
                .get(&source.id)
                .is_some_and(|e| e.source == source && e.watched)
            {
                continue;
            }
            let same_binding = self
                .entries
                .get(&source.id)
                .is_some_and(|e| e.source.generation == source.generation);
            let entry = Entry {
                last: Disk::Image(source.baseline.clone()),
                source: source.clone(),
                watched: false,
            };
            self.entries.insert(source.id, entry);
            let parent = source.path.parent().expect("canonical file has parent");
            let watched = if self.parents.contains_key(parent) {
                Ok(())
            } else {
                let mask = WatchFlags::ATTRIB
                    | WatchFlags::CLOSE_WRITE
                    | WatchFlags::MODIFY
                    | WatchFlags::CREATE
                    | WatchFlags::DELETE
                    | WatchFlags::MOVED_FROM
                    | WatchFlags::MOVED_TO
                    | WatchFlags::DELETE_SELF
                    | WatchFlags::MOVE_SELF
                    | WatchFlags::ONLYDIR
                    | WatchFlags::DONT_FOLLOW;
                inotify::add_watch(&self.descriptor, parent, mask).map(|wd| {
                    self.parents.insert(parent.into(), wd);
                })
            };
            match watched {
                Ok(()) => {
                    self.entries
                        .get_mut(&source.id)
                        .expect("registered source")
                        .watched = true;
                    // Install the watch before observing registration races. A
                    // new baseline is native save/reload evidence, not dismissal.
                    self.schedule(source.id, Instant::now());
                }
                Err(error) => {
                    let disk = Disk::Unavailable(format!(
                        "file observation unavailable: {error}; use revert-buffer"
                    ));
                    self.emit(source.id, disk);
                }
            }
            if !same_binding {
                self.due.remove(&source.id);
                self.schedule(source.id, Instant::now());
            }
        }
    }

    fn request_rescan(&mut self) {
        if self.rescan.is_some() {
            self.rescan_again = true;
        } else {
            self.rescan = Some(0);
        }
    }

    fn schedule(&mut self, id: BufferId, due: Instant) {
        if self.due.contains_key(&id) || self.due.len() < QUEUE {
            self.due.insert(id, due);
        } else {
            self.request_rescan();
        }
    }

    fn events(&mut self) -> io::Result<()> {
        let mut buffer = [MaybeUninit::uninit(); 65_536];
        let mut reader = inotify::Reader::new(&self.descriptor, &mut buffer);
        let mut changes = BTreeSet::new();
        let mut retired = BTreeSet::new();
        let mut lost = false;
        for index in 0..QUEUE {
            let event = match reader.next() {
                Ok(event) => event,
                Err(rustix::io::Errno::AGAIN) => break,
                Err(error) => return Err(error.into()),
            };
            if event.events().intersects(
                ReadFlags::QUEUE_OVERFLOW
                    | ReadFlags::IGNORED
                    | ReadFlags::DELETE_SELF
                    | ReadFlags::MOVE_SELF
                    | ReadFlags::UNMOUNT,
            ) {
                lost = true;
            }
            for (id, entry) in &self.entries {
                if self
                    .parents
                    .get(entry.source.path.parent().expect("file parent"))
                    != Some(&event.wd())
                {
                    continue;
                }
                if event.file_name().is_none_or(|name| {
                    Some(name.to_bytes()) == entry.source.path.file_name().map(|n| n.as_bytes())
                }) {
                    if changes.len() < QUEUE {
                        changes.insert(*id);
                    } else {
                        lost = true;
                    }
                    if event.events().intersects(
                        ReadFlags::IGNORED
                            | ReadFlags::DELETE_SELF
                            | ReadFlags::MOVE_SELF
                            | ReadFlags::UNMOUNT,
                    ) {
                        retired.insert(*id);
                    }
                }
            }
            if event.events().contains(ReadFlags::IGNORED) {
                self.parents.retain(|_, wd| *wd != event.wd());
            }
            if index + 1 == QUEUE {
                lost = true;
            }
        }
        // Discarding the remainder of a bounded kernel batch is an overflow,
        // never evidence that untouched sources remained unchanged.
        for id in retired {
            self.entries.get_mut(&id).expect("retired source").watched = false;
            self.emit(
                id,
                Disk::Unavailable("parent observation was retired; use revert-buffer".into()),
            );
        }
        if lost {
            self.request_rescan();
        }
        let due = Instant::now() + self.debounce;
        for id in changes {
            self.schedule(id, due);
        }
        Ok(())
    }

    fn scan_batch(&mut self) {
        let Some(start) = self.rescan else {
            return;
        };
        if self
            .shared
            .lock()
            .expect("file observation poisoned")
            .events
            .len()
            >= QUEUE
            || self.due.len() >= QUEUE
        {
            return;
        }
        let ids: Vec<_> = self
            .entries
            .keys()
            .skip(start)
            .take(BATCH.min(QUEUE - self.due.len()))
            .copied()
            .collect();
        let end = start + ids.len();
        for id in ids {
            self.schedule(id, Instant::now());
        }
        self.rescan = if end < self.entries.len() {
            Some(end)
        } else if std::mem::take(&mut self.rescan_again) {
            Some(0)
        } else {
            None
        };
    }

    fn check_batch(&mut self) {
        if self
            .shared
            .lock()
            .expect("file observation poisoned")
            .events
            .len()
            >= QUEUE
        {
            return;
        }
        let now = Instant::now();
        let ids: Vec<_> = self
            .due
            .iter()
            .filter(|(_, due)| **due <= now)
            .take(BATCH)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.due.remove(&id);
            let Some(entry) = self.entries.get(&id) else {
                continue;
            };
            if !entry.watched {
                continue;
            }
            let disk = match super::super::observe(&entry.source.path) {
                Ok(observed) => Disk::Image(observed),
                Err(error) => {
                    Disk::Unavailable(format!("file observation: {error}; use revert-buffer"))
                }
            };
            self.emit(id, disk);
        }
    }

    fn emit(&mut self, id: BufferId, disk: Disk) {
        let entry = self
            .entries
            .get_mut(&id)
            .expect("observed source is registered");
        if entry.last == disk {
            return;
        }
        let mut shared = self.shared.lock().expect("file observation poisoned");
        let changed = Changed {
            source: entry.source.clone(),
            disk: disk.clone(),
        };
        if let Some(pending) = shared.events.iter_mut().find(|e| e.source.id == id) {
            *pending = changed;
        } else if shared.events.len() < QUEUE {
            shared.events.push_back(changed);
        } else {
            drop(shared);
            self.request_rescan();
            return;
        }
        entry.last = disk;
        self.wake.notify();
    }

    fn wait(&self) -> Option<Duration> {
        if self
            .shared
            .lock()
            .expect("file observation poisoned")
            .events
            .len()
            >= QUEUE
        {
            return None;
        }
        if self.rescan.is_some() && self.due.len() < QUEUE {
            return Some(Duration::ZERO);
        }
        self.due
            .values()
            .min()
            .map(|due| due.saturating_duration_since(Instant::now()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Controlled overflow covers scheduler backpressure without exhausting the
    // host kernel's shared inotify limits. Real delivery is covered separately.
    #[test]
    fn bounded_rescan() {
        let mut buffers = crate::buffer::Buffers::new().unwrap();
        let mut worker = Worker {
            descriptor: inotify::init(
                inotify::CreateFlags::CLOEXEC | inotify::CreateFlags::NONBLOCK,
            )
            .unwrap(),
            shared: Arc::default(),
            wake: Arc::default(),
            entries: BTreeMap::new(),
            parents: BTreeMap::new(),
            due: BTreeMap::new(),
            rescan: None,
            rescan_again: false,
            debounce: Duration::ZERO,
        };
        for index in 0..300 {
            let id = buffers.create("").unwrap();
            let source = Source {
                id,
                generation: index + 1,
                path: PathBuf::from(format!("/example/source-{index}")),
                baseline: None,
            };
            worker.entries.insert(
                id,
                Entry {
                    source,
                    last: Disk::Image(None),
                    watched: true,
                },
            );
            worker.schedule(id, Instant::now());
        }
        assert_eq!(worker.due.len(), QUEUE);
        assert!(worker.rescan.is_some());
        worker.due.clear();
        let mut seen = BTreeSet::new();
        while worker.rescan.is_some() {
            worker.scan_batch();
            assert!(worker.due.len() <= BATCH);
            seen.extend(worker.due.keys().copied());
            worker.due.clear();
        }
        assert_eq!(seen.len(), 300);
        assert_eq!(
            worker.wait(),
            None,
            "idle observation blocks without a polling deadline"
        );
        let ids: Vec<_> = worker.entries.keys().copied().collect();
        for id in &ids {
            worker.emit(*id, Disk::Unavailable("first observation".into()));
        }
        assert_eq!(worker.shared.lock().unwrap().events.len(), QUEUE);
        assert!(worker.rescan.is_some());
        worker.emit(ids[0], Disk::Unavailable("new observation".into()));
        let state = worker.shared.lock().unwrap();
        assert_eq!(state.events.len(), QUEUE);
        assert_eq!(
            state.events[0].disk,
            Disk::Unavailable("new observation".into())
        );
    }
}
