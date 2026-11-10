mod capture;
mod conflicts;
mod context;
mod controls;
mod envelope;
mod presentation;
mod results;
pub(super) mod save;

use std::path::PathBuf;
use std::time::Duration;

use super::{Action, Session};
pub(super) use crate::ai::target::Kind;
use crate::{
    ai::{pi::Operation, target::Target},
    buffer::Span,
    input::Focus,
};

pub(super) struct Request {
    target: Target,
    definition: crate::config::AiCommand,
    view: crate::window::ViewId,
    home: crate::view::View,
    return_to: crate::location::Anchor,
    root: PathBuf,
    source_name: String,
    language: String,
    conflict: Option<conflicts::Input>,
}

struct Ready {
    request: Request,
    payload: Span,
    generated: String,
    drawer: crate::coordinates::Scalar,
    drawer_body: std::ops::Range<usize>,
}

pub(super) struct Work {
    launch: PathBuf,
    epoch: u64,
    operation: Option<Operation>,
    idle: Option<Operation>,
    query: bool,
    directory: Option<PathBuf>,
    model: Option<String>,
    reset: Option<(PathBuf, Option<String>, u64)>,
    source: Option<Request>,
    ready: Option<Ready>,
    last: Option<crate::config::AiCommand>,
    accepted: bool,
    cleanup_failed: bool,
}

enum Poll {
    Waiting,
    Accepted,
    Finished,
}

struct Completion {
    source: Option<Request>,
    outcome: Option<Result<controls::Outcome, String>>,
}

impl Work {
    pub(in crate::session) fn retains_directory(&self, root: &std::path::Path) -> bool {
        (self.operation.is_some() || self.idle.is_some())
            && (self.directory.as_ref().is_some_and(|p| p.starts_with(root))
                || self
                    .source
                    .as_ref()
                    .is_some_and(|s| s.root.starts_with(root)))
    }
    pub fn new(launch: PathBuf) -> Self {
        Self {
            launch,
            epoch: 0,
            operation: None,
            idle: None,
            query: false,
            directory: None,
            model: None,
            reset: None,
            source: None,
            ready: None,
            last: None,
            accepted: false,
            cleanup_failed: false,
        }
    }
    pub fn stop(&mut self) {
        self.source = None;
        self.operation = None;
        self.idle = None;
    }

    fn poll(&mut self) -> Poll {
        let Some(operation) = self.operation.as_ref() else {
            return Poll::Waiting;
        };
        if unsettled(operation) {
            if operation.accepted() && !operation.is_canceled() && !self.accepted && !self.query {
                self.accepted = true;
                return Poll::Accepted;
            }
            return Poll::Waiting;
        }
        Poll::Finished
    }

    fn complete(&mut self) -> Option<Completion> {
        let operation = self.operation.as_mut().expect("settled Pi operation");
        let epoch = operation.epoch();
        self.cleanup_failed |= operation.cleanup_failed();
        if operation.reset_applied()
            && self
                .reset
                .as_ref()
                .is_some_and(|(_, _, epoch)| *epoch <= operation.epoch())
        {
            self.reset = None;
        }
        let source = self.source.take();
        let query = self.query;
        self.query = false;
        let outcome = if query {
            operation
                .take_models()
                .map(|r| r.map(controls::Outcome::Models))
        } else {
            operation
                .take()
                .map(|r| r.map(controls::Outcome::Generated))
        };
        let operation = self.operation.take().expect("admitted Pi operation");
        if !operation.worker_finished() {
            self.idle = Some(operation);
        }
        if epoch != self.epoch {
            return None;
        }
        Some(Completion { source, outcome })
    }
}

fn unsettled(operation: &Operation) -> bool {
    (operation.is_canceled() && !operation.worker_finished()) || !operation.finished()
}

impl Session {
    pub(super) fn ai_remaining(&self) -> Option<Duration> {
        self.ai
            .operation
            .as_ref()
            .map(|_| Duration::from_millis(10))
    }

    pub(super) fn expire_ai(&mut self) -> Action {
        match self.ai.poll() {
            Poll::Accepted => self.message(
                "Pi accepted; awaiting settled result (guarded directory reads); C-g cancels",
            ),
            Poll::Finished => {
                let Some(completion) = self.ai.complete() else {
                    return Action::Wait;
                };
                self.ai_completed(completion)
            }
            Poll::Waiting => Action::Wait,
        }
    }

    fn ai_completed(&mut self, completion: Completion) -> Action {
        match completion.outcome {
            Some(Ok(controls::Outcome::Generated(output))) => completion
                .source
                .map_or(Action::Wait, |request| self.ai_present(request, output)),
            Some(Ok(controls::Outcome::Models(models))) => self.ai_model_choices(models),
            Some(Err(error)) => self.error(&error),
            None => Action::Wait,
        }
    }

    pub(super) fn cancel_ai(&mut self) -> bool {
        let Some(operation) = &self.ai.operation else {
            return false;
        };
        if operation.is_canceled() {
            return false;
        }
        operation.cancel();
        self.ai.source = None;
        true
    }

    pub(super) fn cancel_ai_input(&mut self) -> Option<Action> {
        if matches!(
            self.pending,
            Some(
                super::Pending::Ai(_)
                    | super::Pending::AiDirectory
                    | super::Pending::AiModel(_)
                    | super::Pending::AiSave(_)
            )
        ) {
            self.pending = None;
            self.input.cancel();
            return Some(self.message("AI instruction canceled; source unchanged"));
        }
        if self.pending.is_some() || self.input.focus() != Focus::Editing {
            return None;
        }
        if self
            .ai
            .ready
            .as_ref()
            .is_some_and(|r| r.payload.stamp.buffer == self.view().point.stamp.buffer)
        {
            return Some(self.ai_discard_result());
        }
        if self.cancel_ai() {
            return Some(self.message("Pi canceled; source retained; stopping owned child"));
        }
        None
    }
}
