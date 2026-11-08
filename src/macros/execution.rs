use super::{Entry, RUN_STEPS, Recorder, Runner};
use crate::buffer::edit::Point;
use crate::window::ViewId;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

pub(crate) const PAUSE_LIMIT: usize = 4;

/// One outer budget follows automatic work through user-entered nested runs.
/// Query, consent and manual editing suspend its clock, without a wake timer.
pub(crate) struct Limits {
    pub steps: AtomicUsize,
    iterations: AtomicUsize,
    pub text: Arc<AtomicUsize>,
    clock: Mutex<Clock>,
}
struct Clock {
    started: Instant,
    paused: Duration,
    since: Option<Instant>,
}
impl Limits {
    pub fn new() -> Self {
        Self {
            steps: AtomicUsize::new(0),
            iterations: AtomicUsize::new(0),
            text: Arc::new(AtomicUsize::new(0)),
            clock: Mutex::new(Clock {
                started: Instant::now(),
                paused: Duration::ZERO,
                since: None,
            }),
        }
    }
    pub fn iteration(&self) -> bool {
        self.iterations
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                (value < RUN_STEPS).then_some(value + 1)
            })
            .is_ok()
    }
    pub fn pause(&self) {
        self.clock
            .lock()
            .expect("macro clock poisoned")
            .since
            .get_or_insert_with(Instant::now);
    }
    pub fn resume(&self) {
        let mut clock = self.clock.lock().expect("macro clock poisoned");
        if let Some(since) = clock.since.take() {
            clock.paused += since.elapsed();
        }
    }
    pub fn elapsed(&self) -> Duration {
        let clock = self.clock.lock().expect("macro clock poisoned");
        clock
            .started
            .elapsed()
            .saturating_sub(clock.paused)
            .saturating_sub(clock.since.map_or(Duration::ZERO, |since| since.elapsed()))
    }
}

pub(crate) struct Pause {
    pub recorder: Option<Recorder>,
    pub runner: Option<Runner>,
    pub query: bool,
    pub expected: (ViewId, Point),
    pub entry: Entry,
    pub waiting: bool,
}

pub(crate) struct Region {
    pub view: ViewId,
    pub anchors: Arc<Mutex<crate::buffer::edit::MacroLines>>,
    pub next: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simulated long user waits are excluded from active automatic time. A
    // resumed child shares that clock; entering another pause is idempotent.
    #[test]
    fn queries_clock() {
        let limits = Limits::new();
        let now = Instant::now();
        {
            let mut clock = limits.clock.lock().unwrap();
            clock.started = now - Duration::from_secs(120);
            clock.since = Some(now - Duration::from_secs(119));
        }
        limits.pause();
        assert!(limits.elapsed() < Duration::from_secs(2));
        limits.resume();
        assert!(limits.elapsed() < Duration::from_secs(2));
        limits.pause();
        assert!(limits.elapsed() < Duration::from_secs(2));
    }
}
