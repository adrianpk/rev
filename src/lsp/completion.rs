//! Completion observations share the connection queue and never own edits.
mod admit;
mod edits;
mod proposal;
pub(crate) use proposal::Proposal;

use super::{Client, Status, documents::Activation, inbox::Inbox};
use crate::buffer::{Buffer, Stamp};
use ropey::Rope;
use std::ops::Range;
use std::sync::{
    Arc, Condvar, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fallback {
    Unavailable,
    Unsupported,
    Empty,
    NoAdmitted,
    Timeout,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Native(Fallback),
    Semantic { incomplete: bool },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Edit {
    pub range: Range<usize>,
    pub text: String,
}

pub(crate) struct Item {
    pub label: String,
    pub kind: Option<u64>,
    pub text: String,
    pub range: Range<usize>,
    pub additional: Vec<Edit>,
    pub proposal: Option<Arc<Proposal>>,
}

pub(crate) struct Reply {
    pub items: Vec<Item>,
    pub source: Source,
    pub version: Option<super::DocumentVersion>,
}
impl Reply {
    pub fn fallback(reason: Fallback) -> Self {
        Self {
            items: Vec::new(),
            source: Source::Native(reason),
            version: None,
        }
    }
}

struct State {
    result: Mutex<Option<Reply>>,
    wake: Condvar,
    cancelled: AtomicBool,
    inbox: Weak<Inbox>,
}

#[derive(Clone)]
pub(crate) struct Ticket(Arc<State>);
impl Ticket {
    fn new(inbox: &Arc<Inbox>) -> Self {
        Self(Arc::new(State {
            result: Mutex::new(None),
            wake: Condvar::new(),
            cancelled: AtomicBool::new(false),
            inbox: Arc::downgrade(inbox),
        }))
    }
    pub fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::Release);
        self.finish(Reply::fallback(Fallback::Cancelled));
        if let Some(inbox) = self.0.inbox.upgrade() {
            inbox.cancel();
        }
    }
    pub fn cancelled(&self) -> bool {
        self.0.cancelled.load(Ordering::Acquire)
    }
    pub fn poll(&self) -> Option<Reply> {
        self.0.result.lock().expect("LSP completion lock").take()
    }
    pub fn wait(&self) -> Reply {
        let result = self.0.result.lock().expect("LSP completion lock");
        let mut result = self
            .0
            .wake
            .wait_while(result, |result| result.is_none())
            .expect("LSP completion wait");
        result.take().expect("completed LSP request")
    }
    pub(super) fn finish(&self, reply: Reply) {
        let mut result = self.0.result.lock().expect("LSP completion lock");
        if result.is_some() {
            return;
        }
        *result = Some(reply);
        drop(result);
        self.0.wake.notify_one();
    }
}

pub(super) struct Query {
    pub ticket: Ticket,
    pub token: Arc<Activation>,
    pub stamp: Stamp,
    pub text: Rope,
    pub point: usize,
    pub range: Range<usize>,
    pub prefix: String,
    pub uri: String,
    pub manual: bool,
    pub resolve: Option<Arc<Proposal>>,
}
impl Query {
    pub fn current(&self) -> bool {
        !self.ticket.cancelled() && self.token.version().is_some_and(|v| v.stamp == self.stamp)
    }
    pub fn finish(&self, result: serde_json::Value, wire: (&super::Encoding, bool)) {
        let incomplete = result
            .get("isIncomplete")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let mut reply = if self.resolve.is_some() {
            admit::resolved(self, result, wire.0)
        } else {
            admit::response(self, &result, wire)
        };
        let Some(version) = self
            .token
            .version()
            .filter(|version| version.stamp == self.stamp)
        else {
            self.ticket.finish(Reply::fallback(Fallback::Cancelled));
            return;
        };
        if self.resolve.is_none() {
            self.token.incomplete.store(incomplete, Ordering::Release);
        }
        if matches!(reply.source, Source::Semantic { .. }) {
            reply.version = Some(version);
        }
        self.ticket.finish(reply);
    }
}
impl Drop for Query {
    fn drop(&mut self) {
        self.ticket.finish(Reply::fallback(Fallback::Unavailable));
    }
}

impl Client {
    pub(crate) fn triggered(&self, id: crate::buffer::BufferId, text: &str) -> bool {
        matches!(self.buffer_status(id), Status::Ready(caps) if caps.triggers.iter().any(|trigger| trigger == text))
    }
    pub(crate) fn completion_possible(&self, id: crate::buffer::BufferId) -> bool {
        matches!(
            self.buffer_status(id),
            Status::Starting
                | Status::Ready(super::Capabilities {
                    completion: true,
                    ..
                })
        )
    }
    pub(crate) fn completion(
        &self,
        buffer: &Buffer,
        target: (usize, Range<usize>, String),
        manual: bool,
    ) -> Option<Ticket> {
        if !self.completion_possible(buffer.id()) {
            return None;
        }
        let attachment = self.attachments.get(&buffer.id())?;
        let connection = self.connections.get(&attachment.key)?;
        let ticket = Ticket::new(&connection.inbox);
        let query = Query {
            ticket: ticket.clone(),
            token: Arc::clone(&attachment.token),
            stamp: buffer.stamp(),
            text: buffer.text_snapshot(),
            point: target.0,
            range: target.1,
            prefix: target.2,
            manual,
            resolve: None,
            uri: super::roots::uri(&attachment.path).ok()?,
        };
        connection.inbox.completion(query);
        Some(ticket)
    }
}

impl Client {
    pub(crate) fn resolve(&self, proposal: Arc<Proposal>) -> Option<Ticket> {
        let context = &proposal.context;
        if self.document_version(context.stamp.buffer) != Some(context.version) {
            return None;
        }
        let attachment = self.attachments.get(&context.stamp.buffer)?;
        let connection = self.connections.get(&attachment.key)?;
        let ticket = Ticket::new(&connection.inbox);
        let query = Query {
            ticket: ticket.clone(),
            token: Arc::clone(&context.token),
            stamp: context.stamp,
            text: context.text.clone(),
            point: context.point,
            range: context.range.clone(),
            prefix: context.prefix.clone(),
            uri: context.uri.clone(),
            manual: true,
            resolve: Some(proposal),
        };
        connection.inbox.completion(query);
        Some(ticket)
    }
}
