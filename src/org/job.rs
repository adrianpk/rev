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
pub struct Cancellation {
    flag: Arc<AtomicBool>,
    deadline: Option<std::time::Instant>,
}

impl Cancellation {
    pub(crate) fn bounded(&self, deadline: std::time::Instant) -> Self {
        Self {
            flag: self.flag.clone(),
            deadline: Some(self.deadline.map_or(deadline, |old| old.min(deadline))),
        }
    }
    pub(crate) fn expired(&self) -> bool {
        self.deadline
            .is_some_and(|end| std::time::Instant::now() >= end)
    }
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Acquire) || self.expired()
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

struct Finished<T> {
    result: Result<(Stamp, T), JobError>,
    cancelled: bool,
}
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
        self.poll_owned().map(|result| {
            result.and_then(|(stamp, value)| {
                super::source::current(buffer, stamp).map_err(JobError::Source)?;
                Ok(value)
            })
        })
    }
    /// Reap a multi-source observation. Its caller must validate every retained
    /// source individually; the initiating view is not the source set's owner.
    pub(crate) fn poll_owned(&mut self) -> Option<Result<(Stamp, T), JobError>> {
        self.finished().map(|finished| {
            if finished.cancelled {
                Err(JobError::Cancelled)
            } else {
                finished.result
            }
        })
    }
    /// File-effect workers must retain their actual outcome even after a late
    /// cancellation. They check cancellation before effects and report each
    /// completed publication; the caller still validates source authority.
    pub(crate) fn poll_effect(&mut self) -> Option<Result<(Stamp, T), JobError>> {
        self.finished().map(|finished| finished.result)
    }
    fn finished(&mut self) -> Option<Finished<T>> {
        if !self.active.as_ref()?.handle.is_finished() {
            return None;
        }
        let active = self.active.take().expect("finished worker owns slot");
        let result = active
            .handle
            .join()
            .map_err(|_| JobError::Panicked)
            .map(|value| (active.stamp, value));
        Some(Finished {
            result,
            cancelled: active.cancellation.is_cancelled(),
        })
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
