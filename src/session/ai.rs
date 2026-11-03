mod capture;
mod context;
mod results;

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
    view: crate::window::ViewId,
    home: crate::view::View,
    return_to: crate::location::Anchor,
    root: PathBuf,
}

struct Ready {
    request: Request,
    payload: Span,
}

pub(super) struct Work {
    launch: PathBuf,
    epoch: u64,
    operation: Option<Operation>,
    source: Option<Request>,
    ready: Option<Ready>,
    accepted: bool,
    cleanup_failed: bool,
}

impl Work {
    pub fn new(launch: PathBuf) -> Self {
        Self {
            launch,
            epoch: 0,
            operation: None,
            source: None,
            ready: None,
            accepted: false,
            cleanup_failed: false,
        }
    }
    pub fn stop(&mut self) {
        self.source = None;
        self.operation = None;
    }
}

impl Session {
    pub(super) fn ai_remaining(&self) -> Option<Duration> {
        self.ai
            .operation
            .as_ref()
            .map(|_| Duration::from_millis(10))
    }

    pub(super) fn expire_ai(&mut self) -> Action {
        let Some(operation) = self.ai.operation.as_mut() else {
            return Action::Wait;
        };
        if !operation.finished() {
            if operation.accepted() && !operation.is_canceled() && !self.ai.accepted {
                self.ai.accepted = true;
                return self.message(
                    "Pi accepted; awaiting settled result (guarded directory reads); C-g cancels",
                );
            }
            return Action::Wait;
        }
        let epoch = operation.epoch();
        let result = operation.take();
        self.ai.cleanup_failed |= operation.cleanup_failed();
        self.ai.operation = None;
        let source = self.ai.source.take();
        if epoch != self.ai.epoch {
            return Action::Wait;
        }
        match result {
            Some(Ok(output)) => {
                let Some(request) = source else {
                    return Action::Wait;
                };
                self.ai_present(request, output)
            }
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
        if matches!(self.pending, Some(super::Pending::Ai(_))) {
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
