//! One owned process generation per package in an editor. Input exchanges only
//! bounded work with supervisors; spawning, pipe I/O and reaping stay outside it.
pub(crate) mod editor;
mod process;
mod worker;

use rev_plugin_sdk::{identity::*, limits::Limits, methods::*};
use rustix::{
    event::{EventfdFlags, eventfd},
    fd::OwnedFd,
};
use std::{
    collections::{BTreeMap, VecDeque},
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        mpsc::{self, SyncSender},
    },
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use super::declarations::Package;
use crate::file::watch::Wake;

static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);
const NOTICES: usize = 32;

pub(crate) struct Service {
    started: Instant,
    remaining: usize,
}
impl Service {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            remaining: 32,
        }
    }
    pub fn take(&mut self) -> bool {
        if self.remaining == 0 || self.started.elapsed() >= Duration::from_millis(2) {
            return false;
        }
        self.remaining -= 1;
        true
    }
}

/// Ephemeral process observation; installation and user buffers have independent
/// lifetimes. Failed packages require a new explicit use or restart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Installed,
    Activating,
    Active,
    Stopping,
    Inactive,
    Failed,
}
impl State {
    pub fn name(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Activating => "activating",
            Self::Active => "active",
            Self::Stopping => "stopping",
            Self::Inactive => "inactive",
            Self::Failed => "failed",
        }
    }
}

/// A bounded copy, with inert diagnostic text rather than terminal instructions.
#[derive(Clone, Debug)]
pub struct Inspection {
    pub state: State,
    pub generation: Option<Generation>,
    pub pending: usize,
    pub pid: Option<u32>,
    pub diagnostic: String,
    pub stderr: String,
    pub stderr_bytes: usize,
    pub cleanup: String,
}
impl Default for Inspection {
    fn default() -> Self {
        Self {
            state: State::Installed,
            generation: None,
            pending: 0,
            pid: None,
            diagnostic: String::new(),
            stderr: String::new(),
            stderr_bytes: 0,
            cleanup: String::new(),
        }
    }
}

#[derive(Default)]
struct Budget {
    pending: AtomicUsize,
    bytes: AtomicUsize,
    ingress: AtomicUsize,
}
fn reserve(counter: &AtomicUsize, count: usize, limit: usize) -> bool {
    counter
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |old| {
            old.checked_add(count).filter(|new| *new <= limit)
        })
        .is_ok()
}

struct Permit {
    budget: Arc<Budget>,
    package: Arc<AtomicUsize>,
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.budget.pending.fetch_sub(1, Ordering::AcqRel);
        self.package.fetch_sub(1, Ordering::AcqRel);
    }
}
struct Call {
    id: OpaqueId,
    command: CommandId,
    context: InvocationContext,
    deadline: Instant,
    _permit: Permit,
}
struct Shared {
    state: State,
    pid: Option<u32>,
    notices: VecDeque<String>,
    diagnostic: String,
    stderr: VecDeque<u8>,
    cleanup: String,
    finished: bool,
    requests: VecDeque<editor::Request>,
}
struct Connection {
    shared: Mutex<Shared>,
    control: OwnedFd,
    stop: AtomicBool,
    ready: AtomicBool,
    pending: Arc<AtomicUsize>,
    calls: SyncSender<Call>,
    responses: SyncSender<editor::Response>,
    wake: Arc<Wake>,
}
impl Connection {
    fn signal(&self) {
        match rustix::io::write(&self.control, &1u64.to_ne_bytes()) {
            Ok(_) | Err(rustix::io::Errno::AGAIN) => {}
            Err(error) => panic!("plugin control descriptor: {error}"),
        }
    }
    fn notify(&self) {
        if !self.ready.swap(true, Ordering::AcqRel) {
            self.wake.signal();
        }
    }
    fn notice(&self, text: String) {
        let mut shared = self.shared.lock().expect("plugin supervisor owner");
        // Completion notices never own execution authority. Retain the newest
        // finite observations when the UI is busy with a native operation.
        if shared.notices.len() == NOTICES {
            shared.notices.pop_front();
        }
        shared.notices.push_back(text);
        drop(shared);
        self.notify();
    }
    fn stop(&self) {
        self.stop.store(true, Ordering::Release);
        self.signal();
    }
}
struct Slot {
    package: Package,
    generation: Generation,
    connection: Arc<Connection>,
    thread: Option<JoinHandle<()>>,
    state: State,
    pid: Option<u32>,
    retired: bool,
    restart: bool,
}

pub(crate) struct Host {
    session: OpaqueId,
    next: u64,
    serial: u64,
    slots: BTreeMap<PackageId, Slot>,
    history: BTreeMap<PackageId, Inspection>,
    budget: Arc<Budget>,
    wake: Arc<Wake>,
    closing: bool,
    cursor: usize,
}
impl Default for Host {
    fn default() -> Self {
        let serial = NEXT_SESSION
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .expect("host session identities exhausted");
        // Process-local counters distinguish simultaneous library hosts; epoch
        // nanoseconds also separate sequential editor lifetimes after PID reuse.
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| e.duration())
            .as_nanos();
        Self {
            session: OpaqueId::new(format!("host-{}-{epoch}-{serial}", std::process::id()))
                .expect("finite host session identity"),
            next: 0,
            serial: 0,
            slots: BTreeMap::new(),
            history: BTreeMap::new(),
            budget: Arc::default(),
            wake: Arc::default(),
            closing: false,
            cursor: 0,
        }
    }
}
impl Host {
    pub fn arm(&self, armed: bool) {
        self.wake.arm(armed);
    }
    pub fn activate(&mut self, package: &Package) -> Result<(), String> {
        if self.closing {
            return Err("Plugin host is shutting down".into());
        }
        if let Some(slot) = self.slots.get(&package.manifest.plugin.id) {
            if !slot.retired
                && !slot.connection.stop.load(Ordering::Acquire)
                && matches!(slot.state, State::Activating | State::Active)
            {
                return Ok(());
            }
            let shared = slot
                .connection
                .shared
                .try_lock()
                .map_err(|_| "Plugin lifecycle update pending; retry activation")?;
            if shared.state == State::Failed && !shared.finished {
                return Err(
                    "Plugin owned group cleanup uncertain; another generation cannot start".into(),
                );
            }
            if !shared.finished {
                return Err("Plugin generation is stopping; wait for its cleanup receipt".into());
            }
        }
        if self
            .slots
            .values()
            .filter(|s| !s.connection.shared.try_lock().is_ok_and(|s| s.finished))
            .count()
            >= 8
        {
            return Err("Plugin active process limit exceeded (8)".into());
        }
        self.next = self
            .next
            .checked_add(1)
            .ok_or("Plugin generations exhausted")?;
        let generation = Generation::new(self.next.to_string()).map_err(|e| e.to_string())?;
        let (calls, incoming) = mpsc::sync_channel(16);
        let (responses, replies) = mpsc::sync_channel(16);
        let connection = Arc::new(Connection {
            shared: Mutex::new(Shared {
                state: State::Activating,
                pid: None,
                notices: VecDeque::new(),
                diagnostic: String::new(),
                stderr: VecDeque::new(),
                cleanup: String::new(),
                finished: false,
                requests: VecDeque::new(),
            }),
            control: eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)
                .map_err(|e| format!("Plugin control unavailable: {e}"))?,
            stop: AtomicBool::new(false),
            ready: AtomicBool::new(false),
            pending: Arc::default(),
            calls,
            responses,
            wake: self.wake.clone(),
        });
        let state = connection.clone();
        let session = self.session.clone();
        let epoch = generation.clone();
        let declaration = package.clone();
        let budget = self.budget.clone();
        let thread = std::thread::Builder::new()
            .name("plugin-supervisor".into())
            .spawn(move || {
                worker::run(
                    declaration,
                    session,
                    epoch,
                    state,
                    budget,
                    incoming,
                    replies,
                )
            })
            .map_err(|e| format!("Plugin supervisor unavailable: {e}"))?;
        self.slots.insert(
            package.manifest.plugin.id.clone(),
            Slot {
                package: package.clone(),
                generation,
                connection,
                thread: Some(thread),
                state: State::Activating,
                pid: None,
                retired: false,
                restart: false,
            },
        );
        Ok(())
    }
    pub fn invoke(
        &mut self,
        package: &Package,
        command: CommandId,
        mut context: InvocationContext,
    ) -> Result<(), String> {
        if self.budget.pending.load(Ordering::Acquire) >= 64 {
            return Err("Plugin pending request limit exceeded (64)".into());
        }
        if self
            .slots
            .get(&package.manifest.plugin.id)
            .is_some_and(|s| s.connection.pending.load(Ordering::Acquire) >= 16)
        {
            return Err("Plugin pending request limit exceeded (16 per package)".into());
        }
        self.activate(package)?;
        let slot = self
            .slots
            .get(&package.manifest.plugin.id)
            .expect("admitted activation");
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or("Plugin request identities exhausted")?;
        let id = OpaqueId::new(format!(
            "{}:{}:{}",
            self.session, slot.generation, self.serial
        ))
        .map_err(|e| e.to_string())?;
        context.generation = slot.generation.clone();
        context.invocation = id.clone();
        context.origin.operation = id.clone();
        context
            .validate(&Limits::default())
            .map_err(|e| e.to_string())?;
        if !reserve(&self.budget.pending, 1, 64) {
            return Err("Plugin pending request limit exceeded (64)".into());
        }
        if !reserve(&slot.connection.pending, 1, 16) {
            self.budget.pending.fetch_sub(1, Ordering::AcqRel);
            return Err("Plugin pending request limit exceeded (16 per package)".into());
        }
        let call = Call {
            id,
            command,
            context,
            deadline: Instant::now() + Duration::from_secs(60),
            _permit: Permit {
                budget: self.budget.clone(),
                package: slot.connection.pending.clone(),
            },
        };
        if slot.connection.stop.load(Ordering::Acquire) {
            return Err("Plugin command unavailable: generation disconnected".into());
        }
        slot.connection
            .calls
            .try_send(call)
            .map_err(|_| "Plugin command queue unavailable")?;
        slot.connection.signal();
        Ok(())
    }
    pub fn context(&self) -> InvocationContext {
        let id = OpaqueId::new("pending").expect("literal identity");
        InvocationContext {
            invocation: id.clone(),
            generation: Generation::new("0").expect("literal generation"),
            stamp: None,
            pane: None,
            point: None,
            selection: None,
            prefix: None,
            origin: Origin {
                package: None,
                operation: id,
                depth: 0,
            },
        }
    }
    pub fn handle(&self, id: String) -> Handle {
        Handle {
            session: self.session.clone(),
            id: OpaqueId::new(id).expect("finite native identity"),
        }
    }
    pub fn deactivate(&mut self, id: &PackageId, restart: bool) -> Result<(), String> {
        let Some(slot) = self.slots.get_mut(id) else {
            return Ok(());
        };
        slot.retired = true;
        slot.restart = restart;
        slot.state = State::Stopping;
        slot.connection.stop();
        slot.connection.notify();
        Ok(())
    }
    pub fn restart(&mut self, package: &Package) -> Result<(), String> {
        if let Some(slot) = self.slots.get(&package.manifest.plugin.id) {
            let shared = slot
                .connection
                .shared
                .try_lock()
                .map_err(|_| "Plugin lifecycle update pending; retry restart")?;
            if shared.state == State::Failed && !shared.finished {
                return Err("Plugin owned group cleanup uncertain; restart is unavailable".into());
            }
            drop(shared);
            self.deactivate(&package.manifest.plugin.id, true)
        } else {
            self.activate(package)
        }
    }
    pub fn cancel(&mut self) -> bool {
        let mut cancelled = false;
        for slot in self.slots.values_mut() {
            if slot.connection.pending.load(Ordering::Acquire) > 0
                || slot.state == State::Activating
            {
                slot.retired = true;
                slot.restart = false;
                slot.state = State::Stopping;
                slot.connection.stop();
                cancelled = true;
            }
        }
        cancelled
    }
    pub fn remaining(&self) -> Option<Duration> {
        self.slots
            .values()
            .any(|s| s.connection.ready.load(Ordering::Acquire))
            .then_some(Duration::ZERO)
    }
    pub fn service(&mut self, service: &mut Service) -> Vec<String> {
        let mut notices = Vec::new();
        let mut restart = Vec::new();
        let mut finished = Vec::new();
        let keys: Vec<_> = self.slots.keys().cloned().collect();
        let cursor = self.cursor;
        for offset in 0..keys.len() {
            let index = (cursor + offset) % keys.len();
            let slot = self.slots.get_mut(&keys[index]).expect("owned plugin slot");
            if !slot.connection.ready.load(Ordering::Acquire) {
                continue;
            }
            if !service.take() {
                break;
            }
            self.cursor = (index + 1) % keys.len();
            let Ok(mut shared) = slot.connection.shared.try_lock() else {
                continue;
            };
            slot.state = if slot.retired && !shared.finished && shared.state != State::Failed {
                State::Stopping
            } else {
                shared.state
            };
            slot.pid = shared.pid;
            while !shared.notices.is_empty() && service.take() {
                let Some(notice) = shared.notices.pop_front() else {
                    break;
                };
                if !slot.retired || shared.finished {
                    notices.push(notice);
                }
            }
            slot.connection.ready.store(
                !shared.notices.is_empty() || !shared.requests.is_empty(),
                Ordering::Release,
            );
            if shared.finished && slot.restart && !self.closing {
                slot.restart = false;
                restart.push(slot.package.clone());
            }
            if shared.finished && shared.notices.is_empty() {
                finished.push(keys[index].clone());
            }
        }
        for id in finished {
            let inspection = self.inspect(&id);
            self.history.insert(id.clone(), inspection);
            self.slots.remove(&id);
        }
        for package in restart {
            if let Err(error) = self.activate(&package) {
                notices.push(error);
            }
        }
        notices
    }
    pub fn owns(&self, handle: &Handle) -> bool {
        handle.session == self.session
    }
    pub fn request(&mut self, service: &mut Service) -> Option<editor::Request> {
        let keys: Vec<_> = self.slots.keys().cloned().collect();
        for offset in 0..keys.len() {
            let index = (self.cursor + offset) % keys.len();
            let slot = self.slots.get(&keys[index]).expect("owned slot");
            let Ok(mut shared) = slot.connection.shared.try_lock() else {
                continue;
            };
            if shared.requests.is_empty() {
                continue;
            }
            if !service.take() {
                return None;
            }
            let request = shared.requests.pop_front().expect("ready editor request");
            self.cursor = (index + 1) % keys.len();
            if !slot.retired && request.live() {
                return Some(request);
            }
        }
        None
    }
    pub fn inspect(&self, id: &PackageId) -> Inspection {
        let Some(slot) = self.slots.get(id) else {
            return self.history.get(id).cloned().unwrap_or_default();
        };
        let mut result = Inspection {
            state: slot.state,
            generation: Some(slot.generation.clone()),
            pending: slot.connection.pending.load(Ordering::Acquire),
            pid: slot.pid,
            ..Inspection::default()
        };
        if let Ok(shared) = slot.connection.shared.try_lock() {
            result.state = if slot.retired && !shared.finished && shared.state != State::Failed {
                State::Stopping
            } else {
                shared.state
            };
            result.pid = shared.pid;
            result.diagnostic = shared.diagnostic.clone();
            result.stderr = inert(&shared.stderr.iter().copied().collect::<Vec<_>>());
            result.stderr_bytes = shared.stderr.len();
            result.cleanup = shared.cleanup.clone();
        }
        result
    }
    pub fn retire(&mut self) {
        self.closing = true;
        for slot in self.slots.values_mut() {
            slot.retired = true;
            slot.restart = false;
            slot.state = State::Stopping;
            slot.connection.stop();
        }
    }
    /// Noninteractive barrier, after restoring the terminal. An unobservable
    /// worker retains ownership; it is never relabeled as successfully reaped.
    pub fn shutdown(&mut self) -> io::Result<()> {
        self.retire();
        let deadline = Instant::now() + Duration::from_secs(5);
        while self
            .slots
            .values()
            .any(|s| s.thread.as_ref().is_some_and(|t| !t.is_finished()))
            && Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(5));
        }
        let mut uncertain = false;
        for slot in self.slots.values_mut() {
            if slot.thread.as_ref().is_some_and(JoinHandle::is_finished) {
                if slot
                    .thread
                    .take()
                    .expect("finished supervisor")
                    .join()
                    .is_err()
                {
                    uncertain = true;
                }
            } else if slot.thread.is_some() {
                uncertain = true;
            }
            let shared = slot.connection.shared.try_lock();
            if !shared.is_ok_and(|s| s.finished) {
                uncertain = true;
            }
        }
        for _ in 0..16 {
            if self.remaining().is_none() {
                break;
            }
            self.service(&mut Service::new());
        }
        if uncertain {
            Err(io::Error::other(
                "Plugin cleanup uncertain; supervisor retains owned resources",
            ))
        } else {
            Ok(())
        }
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        self.retire();
    }
}
fn inert(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .map(|c| {
            if c.is_control() && !matches!(c, '\n' | '\t') {
                '\u{fffd}'
            } else {
                c
            }
        })
        .collect()
}
