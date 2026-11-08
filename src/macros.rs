//! Bounded keyboard capture, pinned playback and explicitly saved definitions.
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::Instant;
mod execution;
pub(crate) use execution::{Limits, PAUSE_LIMIT, Pause, Region};
pub(crate) mod library;
mod names;
pub(crate) use names::{CALL_LIMIT, NAME_LIMIT, Named, Target};

use crate::buffer::edit::Point;
use crate::config::InteractionProfile;
use crate::input::{EditingState, Key};
use crate::window::ViewId;

mod capture;
pub(crate) mod counter;
pub(crate) use capture::{Argument, CommandStep, CompletionEdit, LiteralCompletion, Role};
use counter::Counter;

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
    Query,
    Editing,
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
    CounterOverflow,
    PauseLimit,
    RegionLimit,
    IterationBudget,
    CallCycle,
    CallDepth,
    RetainedMemory,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum QueryMode {
    Confirm,
    Edit,
}

#[derive(Clone)]
pub(crate) enum Step {
    Key(Key),
    Text(String),
    Command(CommandStep),
    Completion(LiteralCompletion),
    Counter(counter::Operation),
    Redisplay,
    Query(QueryMode),
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
            Self::Counter(operation) => operation.bytes(),
            Self::Redisplay | Self::Query(_) => 48,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Definition {
    pub retained_bytes: usize,
    pub id: u64,
    pub revision: u64,
    pub counter: Counter,
    pub initial_format: String,
    pub entry: Entry,
    pub steps: Arc<[Step]>,
}

pub(crate) struct Recorder {
    pub counter: Counter,
    pub initial_format: String,
    pub append: Option<u64>,
    pub original_len: usize,
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
            counter: Counter::default(),
            initial_format: "%d".into(),
            append: None,
            original_len: 0,
            steps: Vec::new(),
            pending: Vec::new(),
            bytes: 4096,
            pending_bytes: 0,
            selecting: false,
            command: None,
        }
    }

    pub fn append(definition: &Definition) -> Self {
        let mut recorder = Self::new(definition.entry);
        recorder.steps = definition.steps.to_vec();
        recorder.bytes += recorder.steps.iter().map(Step::bytes).sum::<usize>();
        recorder.original_len = recorder.steps.len();
        recorder.append = Some(definition.id);
        recorder.counter = definition.counter.clone();
        recorder.initial_format = definition.initial_format.clone();
        recorder
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
    pub name: Option<crate::command::MacroName>,
    pub parent: Option<Box<Runner>>,
    pub counter: Counter,
    pub append: bool,
    pub definition: Arc<Definition>,
    pub iterations: Option<usize>,
    pub iteration: usize,
    pub step: usize,
    pub limits: Arc<Limits>,
    pub query: bool,
    pub region: Option<Region>,
    pub expected: (ViewId, Point),
    pub waiting: bool,
    pub budget: Arc<AtomicUsize>,
    pub group: Arc<crate::buffer::edit::AutomaticGroup>,
    pub outer_group: Option<Arc<crate::buffer::edit::AutomaticGroup>>,
    pub arguments: std::collections::VecDeque<Argument>,
    pub trigger: std::collections::VecDeque<Step>,
    pub consent: Option<Instant>,
}

pub(crate) struct State {
    pub catalog_revision: u64,
    pub library: Option<library::Store>,
    pub names: std::collections::BTreeMap<crate::command::MacroName, Named>,
    pub bindings: std::collections::BTreeMap<crate::input::keymap::Sequence, Target>,
    pub ring: std::collections::VecDeque<Arc<Definition>>,
    next_id: u64,
    pub default_format: String,
    pub scratch: Counter,
    pub last_inserted: i64,
    pub repeat: Option<Repeat>,
    pub recorder: Option<Recorder>,
    pub runner: Option<Runner>,
    pub expanded: bool,
    pub dispatching: bool,
    pub stop: Option<Stop>,
    pub pauses: Vec<Pause>,
}

#[derive(Clone, Copy)]
pub(crate) enum Repeat {
    EndCall,
    Ring,
}

impl Default for State {
    fn default() -> Self {
        Self {
            catalog_revision: 0,
            library: None,
            names: Default::default(),
            bindings: Default::default(),
            ring: Default::default(),
            next_id: 0,
            default_format: "%d".into(),
            scratch: Counter::default(),
            last_inserted: 0,
            repeat: None,
            recorder: None,
            runner: None,
            expanded: false,
            dispatching: false,
            stop: None,
            pauses: Vec::new(),
        }
    }
}

impl State {
    pub fn counter(&self) -> &Counter {
        if let Some(run) = &self.runner {
            &run.counter
        } else if let Some(recorder) = &self.recorder {
            &recorder.counter
        } else if let Some(pause) = self.pauses.last() {
            if let Some(run) = &pause.runner {
                &run.counter
            } else if let Some(recorder) = &pause.recorder {
                &recorder.counter
            } else {
                &self.scratch
            }
        } else {
            self.ring
                .front()
                .map_or(&self.scratch, |definition| &definition.counter)
        }
    }

    pub fn set_counter(&mut self, counter: Counter) {
        if let Some(run) = &mut self.runner {
            run.counter = counter;
        } else if let Some(recorder) = &mut self.recorder {
            recorder.counter = counter;
        } else if let Some(pause) = self.pauses.last_mut() {
            if let Some(run) = &mut pause.runner {
                run.counter = counter;
            } else if let Some(recorder) = &mut pause.recorder {
                recorder.counter = counter;
            } else {
                self.scratch = counter;
            }
        } else if let Some(definition) = self.ring.front_mut() {
            let mut updated = (**definition).clone();
            updated.counter = counter;
            *definition = Arc::new(updated);
        } else {
            self.scratch = counter;
        }
    }

    // Replace metadata by identity; an already admitted runner or snapshot keeps
    // its immutable definition even if the ring is rotated or later edited.
    pub fn finish_run(&mut self) -> Option<Runner> {
        let run = self.runner.take()?;
        self.publish_counter(&run);
        Some(run)
    }

    fn publish_counter(&mut self, run: &Runner) {
        if let Some(definition) = self.ring.iter_mut().find(|d| d.id == run.definition.id) {
            let mut updated = (**definition).clone();
            updated.counter = run.counter.clone();
            *definition = Arc::new(updated);
        }
        for named in self.names.values_mut() {
            if named.definition.id == run.definition.id {
                let mut updated = (*named.definition).clone();
                updated.counter = run.counter.clone();
                named.definition = Arc::new(updated);
            }
        }
        for target in self.bindings.values_mut() {
            if let Target::Snapshot(definition) = target
                && definition.id == run.definition.id
            {
                let mut updated = (**definition).clone();
                updated.counter = run.counter.clone();
                *definition = Arc::new(updated);
            }
        }
    }

    pub fn publish(&mut self, recorder: Recorder, limit: usize) -> Result<(), StopReason> {
        let (id, revision) = if let Some(id) = recorder.append {
            let revision = self
                .ring
                .iter()
                .find(|d| d.id == id)
                .and_then(|d| d.revision.checked_add(1))
                .ok_or(StopReason::CaptureLimit)?;
            (id, revision)
        } else {
            let id = self
                .next_id
                .checked_add(1)
                .ok_or(StopReason::CaptureLimit)?;
            self.next_id = id;
            (id, 1)
        };
        let definition = Arc::new(Definition {
            retained_bytes: recorder.bytes + recorder.steps.len() * std::mem::size_of::<Step>(),
            id,
            revision,
            entry: recorder.entry,
            steps: recorder.steps.into(),
            counter: recorder.counter,
            initial_format: recorder.initial_format,
        });
        self.admit_memory(Some(&definition))?;
        self.ring.retain(|definition| definition.id != id);
        self.ring.push_front(definition);
        self.ring.truncate(limit);
        Ok(())
    }

    pub fn status(&self) -> Status {
        if self.recorder.is_some() {
            Status::Recording
        } else if self.runner.is_some() {
            Status::Playing
        } else if let Some(pause) = self.pauses.last() {
            if pause.recorder.is_some() {
                Status::Recording
            } else {
                Status::Playing
            }
        } else {
            Status::Idle
        }
    }

    pub fn active(&self) -> bool {
        self.recorder.is_some() || self.runner.is_some() || !self.pauses.is_empty()
    }

    pub fn fail(&mut self, reason: StopReason) {
        self.repeat = None;
        let current = self.runner.as_ref().or_else(|| {
            self.pauses
                .iter()
                .rev()
                .find_map(|pause| pause.runner.as_ref())
        });
        self.stop = Some(Stop {
            reason,
            iteration: current.map_or(0, |run| run.iteration + 1),
            step: current.map_or(0, |run| run.step),
        });
        for pause in std::mem::take(&mut self.pauses) {
            let mut run = pause.runner;
            while let Some(frame) = run {
                self.publish_counter(&frame);
                run = frame.parent.map(|parent| *parent);
            }
        }
        self.recorder = None;
        let mut run = self.runner.take();
        while let Some(frame) = run {
            self.publish_counter(&frame);
            run = frame.parent.map(|parent| *parent);
        }
    }
}
