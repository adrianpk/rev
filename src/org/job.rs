use std::fmt;
use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};

use crate::buffer::{self, Buffer, Stamp};

use super::{Index, Scan};

/// Cooperative cancellation, checked between bounded compute or IO steps.
#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);

impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Debug)]
pub enum JobError {
    Busy,
    Spawn(io::Error),
    Cancelled,
    Source(buffer::Error),
    Panicked,
}

impl fmt::Display for JobError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Busy => formatter.write_str("an Org observation is still owned"),
            Self::Spawn(error) => write!(formatter, "start Org observation: {error}"),
            Self::Cancelled => formatter.write_str("Org observation cancelled"),
            Self::Source(error) => error.fmt(formatter),
            Self::Panicked => formatter.write_str("Org observation worker panicked"),
        }
    }
}

impl std::error::Error for JobError {}

struct Running<T> {
    stamp: Stamp,
    cancellation: Cancellation,
    handle: JoinHandle<T>,
}

/// One session-owned slot shared by explicit analysis/IO work. Cancellation does
/// not free the slot until its worker is reaped; no permanent thread is started.
pub struct Job<T> {
    active: Option<Running<T>>,
}

impl<T> Default for Job<T> {
    fn default() -> Self {
        Self { active: None }
    }
}

impl<T> Job<T> {
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Work must check cancellation at bounded compute/read boundaries. Neither
    /// cancellation nor Drop promises to interrupt an already blocked OS read.
    pub fn start<F>(&mut self, stamp: Stamp, work: F) -> Result<(), JobError>
    where
        T: Send + 'static,
        F: FnOnce(&Cancellation) -> T + Send + 'static,
    {
        if self.is_active() {
            return Err(JobError::Busy);
        }
        let cancellation = Cancellation::default();
        let worker_cancel = cancellation.clone();
        let handle = thread::Builder::new()
            .name("org-observation".into())
            .spawn(move || work(&worker_cancel))
            .map_err(JobError::Spawn)?;
        self.active = Some(Running {
            stamp,
            cancellation,
            handle,
        });
        Ok(())
    }

    pub fn cancel(&self) -> bool {
        let Some(active) = &self.active else {
            return false;
        };
        active.cancellation.cancel();
        true
    }

    /// Nonblocking observation. Reap completed ownership before admitting a new
    /// job, and reject obsolete results before exposing them to a caller.
    pub fn poll(&mut self, buffer: &Buffer) -> Option<Result<T, JobError>> {
        if !self.active.as_ref()?.handle.is_finished() {
            return None;
        }
        let active = self.active.take().expect("finished worker owns the slot");
        let result = active.handle.join().map_err(|_| JobError::Panicked);
        Some(result.and_then(|value| {
            if active.cancellation.is_cancelled() {
                return Err(JobError::Cancelled);
            }
            super::source::current(buffer, active.stamp).map_err(JobError::Source)?;
            Ok(value)
        }))
    }
}

impl Job<Index> {
    pub fn start_index(&mut self, scan: Scan) -> Result<(), JobError> {
        self.start(scan.index().stamp(), move |cancel| scan.run(cancel))
    }
}

impl<T> Drop for Job<T> {
    fn drop(&mut self) {
        if let Some(active) = self.active.take() {
            active.cancellation.cancel();
            let _ = active.handle.join();
        }
    }
}
