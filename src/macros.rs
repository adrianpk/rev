//! Bounded session-only keyboard capture and pinned automatic playback.
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

use crate::buffer::edit::Point;
use crate::config::InteractionProfile;
use crate::input::{EditingState, Key};
use crate::window::ViewId;

mod capture;
pub(crate) use capture::{Argument, CommandStep, CompletionEdit, LiteralCompletion, Role};

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

/// Native ownership that suspends automatic input without reusing consent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wait {
    Consent,
    Layout,
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
    MissingCommand,
    PromptMismatch,
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
    Command(CommandStep),
    Completion(LiteralCompletion),
}

impl Step {
    // Reserve a conservative bound on an inert encoded key/text step, including
    // TOML escaping. This also bounds pending prefixes before publication.
    pub(super) fn bytes(&self) -> usize {
        match self {
            Self::Key(_) => 48,
            Self::Text(text) => capture::text_bytes(text),
            Self::Command(command) => command.bytes(),
            Self::Completion(edit) => edit.bytes(),
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
    pub selecting: bool,
    pub command: Option<CommandStep>,
}

impl Recorder {
    pub fn new(entry: Entry) -> Self {
        Self {
            entry,
            steps: Vec::new(),
            pending: Vec::new(),
            bytes: 512,
            pending_bytes: 0,
            selecting: false,
            command: None,
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

    pub fn discard_pending(&mut self) {
        self.pending.clear();
        self.pending_bytes = 0;
    }

    pub fn begin_command(&mut self, mut command: CommandStep, named: bool) {
        if !named {
            command.keys = Some(std::mem::take(&mut self.pending));
        }
        self.discard_pending();
        self.selecting = false;
        self.command = Some(command);
    }

    pub fn argument(&mut self, value: Argument) -> Result<(), StopReason> {
        let command = self.command.as_mut().ok_or(StopReason::PromptMismatch)?;
        if command.arguments.len() >= 32 || value.value.len() > DEFINITION_BYTES {
            return Err(StopReason::CaptureLimit);
        }
        command.arguments.push(value);
        if self.bytes.saturating_add(command.bytes()) > DEFINITION_BYTES {
            return Err(StopReason::CaptureLimit);
        }
        Ok(())
    }

    pub fn finish_command(&mut self) -> Result<(), StopReason> {
        if let Some(command) = self.command.take() {
            self.stage(Step::Command(command))?;
            self.accept()?;
        }
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
    pub arguments: std::collections::VecDeque<Argument>,
    pub trigger: std::collections::VecDeque<Step>,
    pub consent: Option<Instant>,
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
