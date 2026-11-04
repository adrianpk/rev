//! Bounded native interactive sessions. This owner never writes child output to
//! the host terminal and does not create a process until explicitly requested.
mod cleanup;
mod engine;
mod filter;
mod limits;
mod process;
mod readiness;
mod session;
mod viewport;

pub use limits::{
    COMBINING_SCALARS, CONTROL_BYTES, EDITOR_CELLS, EXTRA_BYTES, Geometry, Launch, Limits,
    SERVICE_BYTES, SESSION_CELLS,
};
pub use readiness::Waker;
pub use session::{Session, State, Storage};
use std::{
    collections::BTreeMap,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
pub use viewport::{Selection, Viewport};

/// Stable process/emulator identity, independent of future buffer placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(u64);
impl SessionId {
    pub fn number(self) -> u64 {
        self.0
    }
}

/// One bounded editor-instance registry. Exited sessions retain capacity until
/// explicit closure. Dropping a view cannot remove a session from this owner.
pub struct Sessions {
    sessions: BTreeMap<SessionId, Session>,
    limits: Limits,
    next: u64,
    cursor: usize,
    ready: Arc<AtomicBool>,
    wake: Waker,
}
/// Work performed at one editor service boundary; identities never imply focus.
#[derive(Default, Debug)]
pub struct Service {
    pub bytes: usize,
    pub changed: Vec<SessionId>,
}
impl Sessions {
    pub fn new(limits: Limits, wake: Waker) -> io::Result<Self> {
        limits.validate()?;
        Ok(Self {
            sessions: BTreeMap::new(),
            limits,
            next: 1,
            cursor: 0,
            ready: Arc::new(AtomicBool::new(false)),
            wake,
        })
    }
    pub fn open(&mut self, mut launch: Launch, size: Geometry) -> io::Result<SessionId> {
        launch.validate()?;
        launch.program = launch.executable()?;
        if self.sessions.len() >= self.limits.max_sessions {
            return Err(limits::invalid("Terminal session capacity reached"));
        }
        let cells = size.cells(self.limits.scrollback_lines)?;
        self.admit_cells(cells, None)?;
        let id = SessionId(self.next);
        let next = self
            .next
            .checked_add(1)
            .ok_or_else(|| io::Error::other("Terminal identity exhausted"))?;
        let session = Session::open(
            launch,
            size,
            self.limits.clone(),
            self.ready.clone(),
            self.wake.clone(),
        )?;
        self.sessions.insert(id, session);
        self.next = next;
        Ok(id)
    }
    pub fn get(&self, id: SessionId) -> Option<&Session> {
        self.sessions.get(&id)
    }
    pub fn ids(&self) -> impl Iterator<Item = SessionId> + '_ {
        self.sessions.keys().copied()
    }
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }
    pub fn queue(&mut self, id: SessionId, bytes: &[u8]) -> io::Result<()> {
        self.session(id)?.queue(bytes)
    }
    pub fn resize(&mut self, id: SessionId, size: Geometry) -> io::Result<()> {
        let old = self.sessions.get(&id).ok_or_else(missing)?;
        let cells = old.reservation(size)?;
        self.admit_cells(cells, Some(id))?;
        self.session(id)?.resize(size, cells)
    }
    /// Requests asynchronous close. Service removes the identity only after
    /// successful cleanup; Closing/Failed sessions still consume capacity.
    pub fn close(&mut self, id: SessionId) -> io::Result<()> {
        self.session(id)?.close()?;
        self.ready.store(true, Ordering::Release);
        (self.wake)();
        Ok(())
    }
    /// Batch shutdown outside an interactive UI. Starts all workers before
    /// waiting for their bounded receipts; actual process waits stay on workers.
    /// A failed or pending identity remains owned and consumes capacity.
    pub fn shutdown(&mut self) -> io::Result<()> {
        let ids: Vec<_> = self.ids().collect();
        let mut errors = Vec::new();
        for id in &ids {
            if let Err(error) = self.close(*id) {
                errors.push(error.to_string());
            }
        }
        for id in ids {
            if let Err(error) = self.session(id)?.wait_closed() {
                errors.push(error.to_string());
            }
        }
        self.sessions
            .retain(|_, session| *session.state() != State::Closed);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(io::Error::other(errors.join("; ")))
        }
    }
    /// Nonblocking, fair IO: at most 64 KiB across all sessions. PTY backpressure
    /// retains unread bytes in the kernel; no unbounded pre-parser queue exists.
    pub fn service(&mut self) -> Service {
        self.ready.store(false, Ordering::Release);
        let ids: Vec<_> = self.ids().collect();
        let mut result = Service::default();
        if ids.is_empty() {
            return result;
        }
        let quantum = (SERVICE_BYTES / ids.len()).min(8192);
        for offset in 0..ids.len() {
            let id = ids[(self.cursor + offset) % ids.len()];
            let session = self.sessions.get_mut(&id).expect("registered terminal");
            let (bytes, changed) = session.service(quantum);
            result.bytes += bytes;
            if changed {
                result.changed.push(id);
            }
        }
        self.sessions
            .retain(|_, session| *session.state() != State::Closed);
        self.cursor = (self.cursor + 1) % ids.len();
        result
    }
    fn session(&mut self, id: SessionId) -> io::Result<&mut Session> {
        self.sessions.get_mut(&id).ok_or_else(missing)
    }
    fn admit_cells(&self, cells: usize, replacing: Option<SessionId>) -> io::Result<()> {
        let retained: usize = self
            .sessions
            .iter()
            .filter(|(id, _)| Some(**id) != replacing)
            .map(|(_, session)| session.storage().reserved_cells)
            .sum();
        if retained + cells > EDITOR_CELLS {
            return Err(limits::invalid("Terminal editor cell capacity reached"));
        }
        Ok(())
    }
}
impl Default for Sessions {
    fn default() -> Self {
        Self::new(Limits::default(), Arc::new(|| {})).expect("valid terminal defaults")
    }
}
fn missing() -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        "Terminal session is no longer owned",
    )
}
