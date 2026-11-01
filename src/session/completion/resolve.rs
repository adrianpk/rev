//! Selected-item resolution retains the original invocation until one atomic edit.
use super::{Completion, Session};
use crate::lsp::completion::{Fallback, Source, Ticket};
use crate::session::Action;
use std::time::{Duration, Instant};

pub(in crate::session) struct Resolving {
    completion: Completion,
    ticket: Ticket,
    poll_at: Instant,
}
impl Resolving {
    pub(super) fn cancel(&self) {
        self.ticket.cancel();
    }
    pub(super) fn new(completion: Completion, ticket: Ticket) -> Self {
        Self {
            completion,
            ticket,
            poll_at: Instant::now() + Duration::from_millis(10),
        }
    }
}
impl Session {
    pub(in crate::session) fn completion_remaining(&self, now: Instant) -> Option<Duration> {
        self.completion_work
            .remaining(now)
            .into_iter()
            .chain(
                self.completion_resolve
                    .as_ref()
                    .map(|resolve| resolve.poll_at.saturating_duration_since(now)),
            )
            .min()
    }
    pub(super) fn resolution_current(&self) -> bool {
        self.completion_resolve
            .as_ref()
            .is_some_and(|resolve| self.check_completion_target(&resolve.completion).is_ok())
    }
    pub(super) fn poll_resolution(&mut self, now: Instant) -> Option<Action> {
        let pending = self.completion_resolve.as_mut()?;
        pending.poll_at = now + Duration::from_millis(10);
        let mut reply = pending.ticket.poll()?;
        let mut pending = self
            .completion_resolve
            .take()
            .expect("completed selected resolve");
        if let Err(error) = self.check_completion_target(&pending.completion) {
            self.dismiss_completion();
            return Some(self.error(&error));
        }
        let action = match reply.source {
            Source::Semantic { .. } if reply.items.len() == 1 => {
                if !pending.completion.lsp_current(&self.lsp) {
                    return Some(self.resolve_fallback(Fallback::Unavailable));
                }
                pending.completion.resolved(reply.items.remove(0));
                match self.finish_completion(pending.completion) {
                    Ok(()) => self.message(""),
                    Err(error) => self.error(&error),
                }
            }
            Source::Native(reason) => self.resolve_fallback(reason),
            _ => self.resolve_fallback(Fallback::NoAdmitted),
        };
        Some(action)
    }
    fn resolve_fallback(&mut self, reason: Fallback) -> Action {
        self.dismiss_completion();
        let result = self.capture_completion().map(|mut request| {
            request.native_only = true;
            request.fallback_reason = reason;
            self.completion_work.queue(request);
        });
        match result {
            Ok(()) => self.message("Completion resolve failed; using native fallback"),
            Err(error) => self.error(&error),
        }
    }
}
