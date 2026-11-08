//! Bounded session-only keyboard capture and pinned automatic playback.
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

use crate::buffer::edit::Point;
use crate::config::InteractionProfile;
use crate::input::{EditingState, Key};
use crate::window::ViewId;

pub(crate) const STEP_LIMIT: usize = 4_096;
pub(crate) const DEFINITION_BYTES: usize = 256 * 1024;
pub(crate) const RUN_STEPS: usize = 1_000_000;

/// Observable ownership; idle has no deadline, worker or automatic continuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Recording,
    Playing,
}

/// Typed failure evidence, independent of human-facing message text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    Canceled,
    Empty,
    CaptureLimit,
    Unsupported,
    EntryState,
    OwnerChanged,
    NativeFailure,
    StepBudget,
    TimeBudget,
    TextBudget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stop {
    pub reason: StopReason,
    pub iteration: usize,
    pub step: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    pub profile: InteractionProfile,
    pub mode: Option<EditingState>,
}

#[derive(Clone)]
pub(crate) enum Step {
    Key(Key),
    Text(String),
}

impl Step {
    // Reserve a conservative bound on an inert encoded key/text step, including
    // TOML escaping. This also bounds pending prefixes before publication.
    fn bytes(&self) -> usize {
        match self {
            Self::Key(_) => 48,
            Self::Text(text) if text.len() > DEFINITION_BYTES => usize::MAX,
            Self::Text(text) => toml_edit::Value::from(text.as_str())
                .to_string()
                .len()
                .saturating_add(64),
        }
    }
}

pub(crate) struct Definition {
    pub entry: Entry,
    pub steps: Vec<Step>,
}

pub(crate) struct Recorder {
    pub entry: Entry,
    pub steps: Vec<Step>,
    pending: Vec<Step>,
    bytes: usize,
    pending_bytes: usize,
}

impl Recorder {
    pub fn new(entry: Entry) -> Self {
        Self {
            entry,
            steps: Vec::new(),
            pending: Vec::new(),
            bytes: 512,
            pending_bytes: 0,
        }
    }

    pub fn stage(&mut self, step: Step) -> Result<(), StopReason> {
        let bytes = self.pending_bytes.saturating_add(step.bytes());
        if self.pending.len() >= 32 || bytes > DEFINITION_BYTES {
            return Err(StopReason::CaptureLimit);
        }
        self.pending_bytes = bytes;
        self.pending.push(step);
        Ok(())
    }

    pub fn accept(&mut self) -> Result<(), StopReason> {
        let bytes = self.bytes.saturating_add(self.pending_bytes);
        if self.steps.len() + self.pending.len() > STEP_LIMIT || bytes > DEFINITION_BYTES {
            return Err(StopReason::CaptureLimit);
        }
        self.bytes = bytes;
        self.pending_bytes = 0;
        self.steps.append(&mut self.pending);
        Ok(())
    }
}

pub(crate) struct Runner {
    pub definition: Arc<Definition>,
    pub iterations: usize,
    pub iteration: usize,
    pub step: usize,
    pub dispatched: usize,
    pub started: Instant,
    pub expected: (ViewId, Point),
    pub waiting: bool,
    pub budget: Arc<AtomicUsize>,
    pub group: Arc<crate::buffer::edit::AutomaticGroup>,
}

#[derive(Default)]
pub(crate) struct State {
    pub last: Option<Arc<Definition>>,
    pub recorder: Option<Recorder>,
    pub runner: Option<Runner>,
    pub expanded: bool,
    pub dispatching: bool,
    pub stop: Option<Stop>,
}

impl State {
    pub fn status(&self) -> Status {
        if self.recorder.is_some() {
            Status::Recording
        } else if self.runner.is_some() {
            Status::Playing
        } else {
            Status::Idle
        }
    }

    pub fn active(&self) -> bool {
        self.recorder.is_some() || self.runner.is_some()
    }

    pub fn fail(&mut self, reason: StopReason) {
        if self.active() {
            self.stop = Some(Stop {
                reason,
                iteration: self.runner.as_ref().map_or(0, |r| r.iteration + 1),
                step: self.runner.as_ref().map_or(0, |r| r.step),
            });
            self.recorder = None;
            self.runner = None;
        }
    }
}
