use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::source::{MAX_PAYLOAD, MAX_RETAINED, Source};
use crate::buffer::BufferId;

use super::{Request, Symbol, WORK_TIME};

const POLL: Duration = Duration::from_millis(10);

pub(crate) struct Output {
    pub request: Request,
    pub sources: Vec<Source>,
    pub result: Result<super::Completion, String>,
}

struct Running {
    request: Request,
    cancel: Arc<AtomicBool>,
    handle: JoinHandle<Output>,
    sources: Vec<BufferId>,
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
            .is_some_and(|r| r.sources.contains(&buffer))
            || self
                .pending
                .as_ref()
                .is_some_and(|r| r.point.stamp.buffer == buffer)
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
    pub fn start(&mut self, snapshot: Vec<Source>, now: Instant) -> Result<(), String> {
        let request = self.pending.take().ok_or("No native request is pending")?;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let worker_request = request.clone();
        let sources = snapshot
            .iter()
            .map(|source| source.observation.stamp.buffer)
            .collect();
        let handle = thread::Builder::new()
            .name("native-completion".into())
            .spawn(move || run(worker_request, snapshot, &worker_cancel))
            .map_err(|e| e.to_string())?;
        self.active = Some(Running {
            request,
            cancel,
            handle,
            sources,
        });
        self.poll_at = Some(now + POLL);
        Ok(())
    }
    pub fn poll(&mut self, now: Instant) -> Option<(Request, Result<Output, String>)> {
        let active = self.active.as_ref()?;
        if !active.handle.is_finished() {
            self.poll_at = Some(now + POLL);
            return None;
        }
        self.poll_at = None;
        let active = self.active.take().expect("finished native worker");
        Some((
            active.request,
            active
                .handle
                .join()
                .map_err(|_| "Native completion worker panicked".into()),
        ))
    }
}

fn run(mut request: Request, mut sources: Vec<Source>, cancel: &Arc<AtomicBool>) -> Output {
    let deadline = Instant::now() + WORK_TIME;
    let result = extract(&mut request, &mut sources, deadline, cancel)
        .map(|symbols| request.clone().complete(symbols))
        .and_then(|completion| {
            if cancel.load(Ordering::Acquire) || Instant::now() >= deadline {
                Err("Native completion work limit reached".into())
            } else {
                Ok(completion)
            }
        });
    Output {
        request,
        sources,
        result,
    }
}

fn extract(
    request: &mut Request,
    sources: &mut [Source],
    deadline: Instant,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<Symbol>, String> {
    if let Some(scan) = sources.first_mut().and_then(|source| source.org.take()) {
        let cancellation = crate::org::Cancellation::shared(cancel.clone(), deadline);
        return request.org(scan, &sources[0].text, &cancellation);
    }
    let mut symbols = Vec::new();
    let mut payload = 0;
    for (origin, source) in sources.iter_mut().enumerate() {
        parse(source, deadline, cancel);
        if !prepare_source(request, source, origin, deadline, cancel)? {
            return Ok(Vec::new());
        }
        let result = source.syntax.declarations_with(
            source.text.slice(..),
            source.observation.stamp,
            deadline,
            cancel,
        );
        if origin == 0 {
            append(&mut symbols, result?, origin, &mut payload);
        } else {
            match result {
                Ok(found) if fits(&symbols, &found, payload) => {
                    append(&mut symbols, found, origin, &mut payload)
                }
                _ => request.limited = true,
            }
        }
    }
    let catalog = request.context.catalog();
    if fits(&symbols, &catalog, payload) {
        append(&mut symbols, catalog, usize::MAX, &mut payload);
    } else {
        request.limited = true;
    }
    Ok(symbols)
}

fn fits(symbols: &[Symbol], found: &[Symbol], payload: usize) -> bool {
    symbols.len() + found.len() <= MAX_RETAINED && payload + weight(found) <= MAX_PAYLOAD
}

fn weight(symbols: &[Symbol]) -> usize {
    symbols
        .iter()
        .map(|symbol| std::mem::size_of::<Symbol>() + symbol.name.capacity())
        .sum()
}

fn append(symbols: &mut Vec<Symbol>, mut found: Vec<Symbol>, origin: usize, payload: &mut usize) {
    *payload += weight(&found);
    for symbol in &mut found {
        symbol.origin = origin;
    }
    symbols.extend(found);
}

fn parse(source: &mut Source, deadline: Instant, cancel: &AtomicBool) {
    let mut progress = |_: &tree_sitter::ParseState| {
        if cancel.load(Ordering::Acquire) || Instant::now() >= deadline {
            std::ops::ControlFlow::Break(())
        } else {
            std::ops::ControlFlow::Continue(())
        }
    };
    source.syntax.update_with(
        source.text.slice(..),
        source.observation.stamp,
        tree_sitter::ParseOptions::new().progress_callback(&mut progress),
    );
}

impl Drop for Work {
    fn drop(&mut self) {
        self.cancel();
        if let Some(active) = self.active.take() {
            let _ = active.handle.join();
        }
    }
}

fn prepare_source(
    request: &mut Request,
    source: &Source,
    origin: usize,
    deadline: Instant,
    cancel: &AtomicBool,
) -> Result<bool, String> {
    if origin != 0 {
        return Ok(true);
    }
    request.prepare(&source.syntax, source.text.slice(..));
    request.allowed(source, deadline, cancel)
}
