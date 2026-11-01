use std::collections::VecDeque;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use super::{
    budget::{Budget, Permit},
    completion::Query,
    documents,
    frame::Message,
};

pub(super) const QUEUE_BYTES: usize = 16 * 1024 * 1024;
pub(super) const QUEUE_MESSAGES: usize = 64;

#[derive(Default)]
struct Pending {
    messages: VecDeque<(Message, usize)>,
    documents: VecDeque<(Work, Permit)>,
    bytes: usize,
    failure: Option<String>,
    stop: bool,
    cancelled: bool,
    closed: bool,
}

/// Failures and shutdown have reserved control slots, even under backpressure.
#[derive(Default)]
pub(super) struct Inbox {
    pending: Mutex<Pending>,
    wake: Condvar,
    pub budget: Arc<Budget>,
    closed: AtomicBool,
}

pub(super) enum Event {
    Message(Box<Message>),
    Document(Box<documents::Event>, Permit),
    Completion(Box<Query>, Permit),
    Cancelled,
    Failed(String),
    Stop,
    Timeout,
}

enum Work {
    Document(documents::Event),
    Completion(Query),
}

impl Inbox {
    pub fn cancel(&self) {
        self.pending.lock().expect("LSP inbox lock").cancelled = true;
        self.wake.notify_one();
    }
    pub fn completion(&self, query: Query) {
        let mut state = self.pending.lock().expect("LSP inbox lock");
        if state.closed {
            return;
        }
        state.documents.retain(|(work, _)| !matches!(work, Work::Completion(old) if old.stamp.buffer == query.stamp.buffer));
        let permit = match self.budget.reserve(query.text.len_bytes()) {
            Ok(permit) => permit,
            Err(_) => return,
        };
        state.documents.push_back((Work::Completion(query), permit));
        self.wake.notify_one();
    }

    pub fn push(&self, message: Message, bytes: usize) {
        let mut state = self.pending.lock().expect("LSP inbox lock");
        if state.messages.len() >= QUEUE_MESSAGES || state.bytes + bytes > QUEUE_BYTES {
            state.failure = Some("LSP inbound queue overflow".into());
        } else {
            state.bytes += bytes;
            state.messages.push_back((message, bytes));
        }
        self.wake.notify_one();
    }

    pub fn fail(&self, error: String) {
        self.pending.lock().expect("LSP inbox lock").failure = Some(error);
        self.wake.notify_one();
    }

    pub fn stop(&self) {
        self.pending.lock().expect("LSP inbox lock").stop = true;
        self.wake.notify_one();
    }

    pub fn document(&self, event: documents::Event) {
        let permit = match self.budget.reserve(event.bytes()) {
            Ok(permit) => permit,
            Err(error) => {
                event.retire();
                self.fail(error);
                return;
            }
        };
        let mut pending = self.pending.lock().expect("LSP inbox lock");
        if pending.closed {
            event.retire();
            return;
        }
        pending.documents.push_back((Work::Document(event), permit));
        self.wake.notify_one();
    }

    pub fn close(&self) {
        let mut pending = self.pending.lock().expect("LSP inbox lock");
        pending.closed = true;
        self.closed.store(true, Ordering::Release);
        for (work, _) in &pending.documents {
            if let Work::Document(event) = work {
                event.retire();
            }
        }
        pending.documents.clear();
        pending.messages.clear();
        pending.bytes = 0;
    }

    pub fn closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    pub fn next(&self, timeout: Option<Duration>, ready: bool) -> Event {
        let state = self.pending.lock().expect("LSP inbox lock");
        let idle = |p: &mut Pending| p.idle(ready);
        let mut state = match timeout {
            Some(timeout) => {
                self.wake
                    .wait_timeout_while(state, timeout, idle)
                    .expect("LSP inbox wait")
                    .0
            }
            None => self.wake.wait_while(state, idle).expect("LSP inbox wait"),
        };
        if state.stop {
            state.stop = false;
            return Event::Stop;
        }
        if let Some(error) = state.failure.take() {
            return Event::Failed(error);
        }
        if state.cancelled {
            state.cancelled = false;
            return Event::Cancelled;
        }
        match state.messages.pop_front() {
            Some((message, bytes)) => {
                state.bytes -= bytes;
                Event::Message(Box::new(message))
            }
            None => self.document_event(&mut state, ready),
        }
    }
    fn document_event(&self, state: &mut Pending, ready: bool) -> Event {
        if !ready {
            return Event::Timeout;
        }
        match state.documents.pop_front() {
            Some((Work::Document(event), permit)) => Event::Document(Box::new(event), permit),
            Some((Work::Completion(query), permit)) => Event::Completion(Box::new(query), permit),
            None => Event::Timeout,
        }
    }
}

impl Pending {
    fn idle(&self, ready: bool) -> bool {
        self.messages.is_empty()
            && (!ready || self.documents.is_empty())
            && self.failure.is_none()
            && !self.stop
            && !self.cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Inbound overload preserves reserved stop/failure slots, and byte/count
    // limits apply independently rather than allowing zero-byte message floods.
    #[test]
    fn reserved_controls() {
        for (bytes, count) in [(0, QUEUE_MESSAGES), (QUEUE_BYTES, 1)] {
            let inbox = Inbox::default();
            for _ in 0..count {
                inbox.push(
                    serde_json::from_value(json!({"jsonrpc":"2.0","method":"notice"})).unwrap(),
                    bytes,
                );
            }
            inbox.push(
                serde_json::from_value(json!({"jsonrpc":"2.0","method":"notice"})).unwrap(),
                1,
            );
            inbox.stop();
            assert!(matches!(inbox.next(None, true), Event::Stop));
            assert!(matches!(inbox.next(None, true), Event::Failed(_)));
            inbox.close();
        }
    }
}
