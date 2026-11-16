use std::cell::Cell;
use std::time::{Duration, Instant};

use rev::command::Command;
use rev::display::Frame;

use crate::rendering::{self, marks};

#[path = "pulse/input.rs"]
mod input;
#[path = "pulse/invalidation.rs"]
mod invalidation;
#[path = "pulse/lifetime.rs"]
mod lifetime;

thread_local! {
    static CLOCK: Cell<Instant> = Cell::new(Instant::now());
}

pub(super) fn now() -> Instant {
    CLOCK.with(Cell::get)
}
pub(super) fn reset() {
    CLOCK.with(|clock| clock.set(Instant::now()));
}
pub(super) fn advance(millis: usize) {
    CLOCK.with(|clock| {
        clock.set(clock.get() + Duration::from_millis(u64::try_from(millis).unwrap()))
    });
}

pub(super) struct Editor {
    pub native: rendering::Editor,
}

impl Editor {
    pub fn new(duration: usize, source: &str, point: usize) -> Self {
        Self::build(duration, source, point, true, "non-modal")
    }
    pub fn editable(duration: usize, source: &str, point: usize) -> Self {
        Self::build(duration, source, point, false, "non-modal")
    }
    pub fn profile(profile: &str, duration: usize, source: &str, point: usize) -> Self {
        Self::build(duration, source, point, false, profile)
    }
    fn build(duration: usize, source: &str, point: usize, readonly: bool, profile: &str) -> Self {
        let native = rendering::Editor::prepared(
            &format!(
                "pairs.highlight='pulse'\npairs.pulse_ms={duration}\neditor.soft_wrap=false\neditor.interaction_profile='{profile}'"
            ),
            source,
            point,
            |buffer, _| buffer.set_readonly(readonly),
        );
        Self::attach(native)
    }
    pub fn attach(mut native: rendering::Editor) -> Self {
        reset();
        native.session = native.session.with_pair_clock(now);
        Self { native }
    }
    pub fn frame(&mut self) -> Frame {
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(Instant::now() < end, "finite pulse qualification deadline");
            self.native.session.expire_input(now());
            let frame = self.native.session.frame().unwrap();
            assert!(
                self.native.session.buffer().unwrap().retained_pair_bytes()
                    <= rev::pairs::MAX_METADATA
            );
            if self.remaining().is_none_or(|wait| !wait.is_zero()) {
                return frame;
            }
            std::thread::yield_now();
        }
    }
    pub fn remaining(&self) -> Option<Duration> {
        self.native.session.remaining_input(now())
    }
    pub fn jump(&mut self) {
        self.native.session.execute(Command::MatchingDelimiter);
    }
    pub fn sources(&mut self) -> Vec<usize> {
        marks(&self.frame()).into_iter().map(|m| m.0).collect()
    }
}
