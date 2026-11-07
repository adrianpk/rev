//! Owned, bounded parent-directory notifications. Filesystem work stays on the
//! worker; the session exchanges only source identities and disk observations.
mod worker;

use std::collections::VecDeque;
use std::io;
use std::path::PathBuf;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;

use rustix::event::{EventfdFlags, eventfd};
use rustix::fd::OwnedFd;

use super::Observation;
use crate::buffer::BufferId;

pub(crate) const QUEUE: usize = 256;
pub(crate) const BATCH: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Source {
    pub id: BufferId,
    pub generation: u64,
    pub path: PathBuf,
    pub baseline: Option<Observation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Disk {
    Image(Option<Observation>),
    Unavailable(String),
}

#[derive(Clone)]
pub(crate) struct Changed {
    pub source: Source,
    pub disk: Disk,
}

/// SIGWINCH wakes Crossterm's existing native signal queue only while a terminal
/// session is armed. Equal-size wakeups never become resize/editing transitions.
/// Library sessions inspect the flag without signaling their host.
#[derive(Default)]
pub(crate) struct Wake {
    ready: AtomicBool,
    armed: AtomicBool,
}
impl Wake {
    /// A replacement inherits host signal permission, never a retired source's
    /// ready flag. Each context owns its own notification lifetime.
    pub(crate) fn replacement(&self) -> Arc<Self> {
        Arc::new(Self {
            ready: AtomicBool::new(false),
            armed: AtomicBool::new(self.armed.load(Ordering::Acquire)),
        })
    }
    pub fn ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }
    pub fn arm(&self, armed: bool) {
        self.armed.store(armed, Ordering::Release);
        if armed && self.ready() {
            self.signal();
        }
    }
    fn notify(&self) {
        if !self.ready.swap(true, Ordering::AcqRel) {
            self.signal();
        }
    }
    pub(crate) fn signal(&self) {
        if self.armed.load(Ordering::Acquire) {
            // Exclusively this editor process, using its installed terminal
            // signal queue. No other process or synthesized input is involved.
            let result = rustix::process::kill_process(
                rustix::process::getpid(),
                rustix::process::Signal::WINCH,
            );
            assert!(result.is_ok(), "live editor accepts its native wake signal");
        }
    }
}

#[derive(Default)]
struct Shared {
    update: Option<Vec<Source>>,
    stop: bool,
    events: VecDeque<Changed>,
    diagnostic: Option<String>,
}

pub(crate) struct Watch {
    shared: Arc<Mutex<Shared>>,
    control: Arc<OwnedFd>,
    wake: Arc<Wake>,
    thread: Option<JoinHandle<()>>,
}
impl Watch {
    pub fn start(sources: Vec<Source>, debounce_ms: usize, wake: Arc<Wake>) -> io::Result<Self> {
        let control = Arc::new(eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)?);
        let shared = Arc::new(Mutex::new(Shared {
            update: Some(sources),
            ..Shared::default()
        }));
        let state = shared.clone();
        let signal = control.clone();
        let notify = wake.clone();
        let thread = std::thread::Builder::new()
            .name("file-observation".into())
            .spawn(move || {
                if let Err(error) = worker::run(state.clone(), signal, notify.clone(), debounce_ms)
                {
                    state.lock().expect("file observation poisoned").diagnostic = Some(format!(
                        "file observation unavailable: {error}; use revert-buffer"
                    ));
                    notify.notify();
                }
            })?;
        Ok(Self {
            shared,
            control,
            wake,
            thread: Some(thread),
        })
    }
    pub fn update(&self, sources: Vec<Source>) {
        self.shared
            .lock()
            .expect("file observation poisoned")
            .update = Some(sources);
        signal(&self.control);
    }
    /// A checked edit can reject a queued source replacement without consuming
    /// notifications or taking reload/discard authority from the native owner.
    pub(crate) fn pending_for(&self, id: BufferId) -> bool {
        self.shared
            .lock()
            .expect("file observation poisoned")
            .events
            .iter()
            .any(|event| event.source.id == id)
    }

    pub fn take(&self) -> (Vec<Changed>, Option<String>) {
        let mut shared = self.shared.lock().expect("file observation poisoned");
        let events = (0..BATCH)
            .filter_map(|_| shared.events.pop_front())
            .collect();
        let diagnostic = shared.diagnostic.take();
        self.wake
            .ready
            .store(!shared.events.is_empty(), Ordering::Release);
        drop(shared);
        signal(&self.control);
        (events, diagnostic)
    }
}
impl Drop for Watch {
    fn drop(&mut self) {
        self.wake.arm(false);
        self.shared.lock().expect("file observation poisoned").stop = true;
        signal(&self.control);
        // Never block input on an OS metadata call. Stopped work owns its
        // remaining descriptors and cannot publish into another session.
        if self.thread.as_ref().is_some_and(|t| t.is_finished()) {
            self.thread
                .take()
                .expect("finished observer")
                .join()
                .expect("observer worker failed");
        }
    }
}
fn signal(fd: &OwnedFd) {
    match rustix::io::write(fd, &1u64.to_ne_bytes()) {
        Ok(_) | Err(rustix::io::Errno::AGAIN) => {}
        Err(error) => panic!("owned observation control descriptor failed: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    // Dropping the owner wakes an indefinitely waiting worker and closes its
    // descriptors without leaving an input consumer or another observer alive.
    #[test]
    fn owned_shutdown() {
        let mut watch = Watch::start(Vec::new(), 0, Arc::default()).unwrap();
        let thread = watch.thread.take().unwrap();
        drop(watch);
        let deadline = Instant::now() + Duration::from_secs(2);
        while !thread.is_finished() {
            assert!(Instant::now() < deadline, "observer did not retire");
            std::thread::sleep(Duration::from_millis(1));
        }
        thread.join().unwrap();
    }
}
