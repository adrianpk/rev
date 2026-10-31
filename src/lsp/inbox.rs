use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::time::Duration;

use super::frame::Message;

pub(super) const QUEUE_BYTES: usize = 16 * 1024 * 1024;
pub(super) const QUEUE_MESSAGES: usize = 64;

#[derive(Default)]
struct Pending {
    messages: VecDeque<(Message, usize)>,
    bytes: usize,
    failure: Option<String>,
    stop: bool,
}

/// Failures and shutdown have reserved control slots, even under backpressure.
#[derive(Default)]
pub(super) struct Inbox {
    pending: Mutex<Pending>,
    wake: Condvar,
}

pub(super) enum Event {
    Message(Message),
    Failed(String),
    Stop,
    Timeout,
}

impl Inbox {
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

    pub fn next(&self, timeout: Option<Duration>) -> Event {
        let state = self.pending.lock().expect("LSP inbox lock");
        let idle = |p: &mut Pending| p.messages.is_empty() && p.failure.is_none() && !p.stop;
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
        match state.messages.pop_front() {
            Some((message, bytes)) => {
                state.bytes -= bytes;
                Event::Message(message)
            }
            None => Event::Timeout,
        }
    }
}
