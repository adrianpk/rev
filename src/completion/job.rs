use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::buffer::BufferId;
use crate::syntax::Syntax;

use super::{Request, Symbol, WORK_TIME};

const POLL: Duration = Duration::from_millis(10);

pub(crate) struct Output {
    pub request: Request,
    pub syntax: Syntax,
    pub result: Result<Vec<Symbol>, String>,
}

struct Running {
    request: Request,
    cancel: Arc<AtomicBool>,
    handle: JoinHandle<Output>,
}

/// The slot stays owned until its cancelled worker is reaped. Only the newest
/// pending context survives a burst; there is no permanent analysis thread.
#[derive(Default)]
pub(crate) struct Work {
    active: Option<Running>,
    pub pending: Option<Request>,
    poll_at: Option<Instant>,
}

impl Work {
    pub fn cancel(&mut self) {
        self.pending = None;
        if let Some(active) = &self.active {
            active.cancel.store(true, Ordering::Release);
        }
    }
    pub fn queue(&mut self, request: Request) {
        self.cancel();
        self.pending = Some(request);
    }
    pub fn current(&self, view: crate::window::ViewId, point: crate::buffer::edit::Point) -> bool {
        self.pending
            .as_ref()
            .or_else(|| self.active.as_ref().map(|r| &r.request))
            .is_some_and(|r| r.current(view, point))
    }
    pub fn owns(&self, buffer: BufferId) -> bool {
        self.active
            .as_ref()
            .is_some_and(|r| r.request.point.stamp.buffer == buffer)
    }
    pub fn ready(&self, now: Instant) -> bool {
        self.active.is_none()
            && self
                .pending
                .as_ref()
                .is_some_and(|request| now >= request.due)
    }
    pub fn remaining(&self, now: Instant) -> Option<Duration> {
        let deadline = if self.active.is_some() {
            self.poll_at
        } else {
            self.pending.as_ref().map(|r| r.due)
        };
        deadline.map(|deadline| deadline.saturating_duration_since(now))
    }
    pub fn start(&mut self, snapshot: (ropey::Rope, Syntax), now: Instant) -> Result<(), String> {
        let request = self.pending.take().ok_or("No native request is pending")?;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let worker_request = request.clone();
        let handle = thread::Builder::new()
            .name("native-completion".into())
            .spawn(move || run(worker_request, snapshot, &worker_cancel))
            .map_err(|e| e.to_string())?;
        self.active = Some(Running {
            request,
            cancel,
            handle,
        });
        self.poll_at = Some(now + POLL);
        Ok(())
    }
    pub fn poll(&mut self, now: Instant) -> Option<Result<Output, String>> {
        let active = self.active.as_ref()?;
        if !active.handle.is_finished() {
            self.poll_at = Some(now + POLL);
            return None;
        }
        self.poll_at = None;
        let active = self.active.take().expect("finished native worker");
        Some(
            active
                .handle
                .join()
                .map_err(|_| "Native completion worker panicked".into()),
        )
    }
}

fn run(request: Request, snapshot: (ropey::Rope, Syntax), cancel: &AtomicBool) -> Output {
    let (text, mut syntax) = snapshot;
    let deadline = Instant::now() + WORK_TIME;
    let mut progress = |_: &tree_sitter::ParseState| {
        if cancel.load(Ordering::Acquire) || Instant::now() >= deadline {
            std::ops::ControlFlow::Break(())
        } else {
            std::ops::ControlFlow::Continue(())
        }
    };
    syntax.update_with(
        text.slice(..),
        request.point.stamp,
        tree_sitter::ParseOptions::new().progress_callback(&mut progress),
    );
    let result = syntax.declarations_with(text.slice(..), request.point.stamp, deadline, cancel);
    Output {
        request,
        syntax,
        result,
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        self.cancel();
        if let Some(active) = self.active.take() {
            let _ = active.handle.join();
        }
    }
}
