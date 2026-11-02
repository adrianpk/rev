use std::time::{Duration, Instant};

use crate::buffer::Stamp;
use crate::config::{Publication, Roots};
use crate::org::{Cancellation, Job};

use super::{Installation, Storage};

pub struct Outcome {
    pub installation: Installation,
    pub selection: Option<Publication>,
    pub message: String,
}

/// Reuses the native job owner. No thread, timer or lock exists while idle.
/// A cancelled slot stays owned until its worker and helpers have finished.
#[derive(Default)]
pub struct Manager {
    job: Job<Result<Outcome, String>>,
    progress: super::progress::Progress,
}

impl Manager {
    pub fn phase(&self) -> super::Phase {
        self.progress.phase()
    }

    pub fn active(&self) -> bool {
        self.job.is_active()
    }

    /// Work must check the supplied cancellation token between bounded steps.
    /// Subprocess owners must enforce it while IO is in flight.
    pub fn start<F>(&mut self, source: Stamp, roots: Roots, work: F) -> Result<(), String>
    where
        F: FnOnce(Storage, &Cancellation) -> Result<Outcome, String> + Send + 'static,
    {
        if self.active() {
            return Err("another server management job is still owned".into());
        }
        let progress = self.progress.clone();
        progress.set(super::Phase::Checking);
        self.job
            .start(source, move |cancel| {
                let cancel =
                    cancel.bounded(Instant::now() + Duration::from_secs(super::OPERATION_SECONDS));
                let result = run(roots, &progress, &cancel, work);
                progress.set(terminal(&result, &cancel));
                result
            })
            .map_err(|e| {
                self.progress.set(super::Phase::Failed);
                e.to_string()
            })
    }

    pub fn cancel(&self) -> bool {
        self.job.cancel()
    }

    /// Effects retain their committed outcome after late cancellation. Buffer
    /// changes cannot invalidate already published installation facts.
    pub fn poll(&mut self) -> Option<Result<Outcome, String>> {
        self.job.poll_effect().map(|result| {
            result
                .map_err(|e| {
                    self.progress.set(super::Phase::Failed);
                    e.to_string()
                })
                .and_then(|(_, outcome)| outcome)
        })
    }
}

fn run<F>(
    roots: Roots,
    progress: &super::progress::Progress,
    cancel: &Cancellation,
    work: F,
) -> Result<Outcome, String>
where
    F: FnOnce(Storage, &Cancellation) -> Result<Outcome, String>,
{
    let mut storage = Storage::acquire(roots)?;
    storage.progress = progress.clone();
    if cancel.is_cancelled() {
        return Err("installation cancelled before acquisition".into());
    }
    work(storage, cancel)
}

fn terminal(result: &Result<Outcome, String>, cancel: &Cancellation) -> super::Phase {
    if result.is_ok() {
        super::Phase::Ready
    } else if cancel.is_cancelled() && !cancel.expired() {
        super::Phase::Cancelled
    } else {
        super::Phase::Failed
    }
}
