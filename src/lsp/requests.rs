//! One pending completion per buffer; connection-wide finite admission/deadlines.
use super::{
    Capabilities, Encoding,
    budget::Permit,
    completion::{Fallback, Query, Reply},
    frame::Message,
    io::Outbox,
    position,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

struct Pending {
    query: Query,
    deadline: Instant,
    encoding: Encoding,
}
pub(super) struct Requests {
    pending: BTreeMap<u64, Pending>,
    next: u64,
    limit: usize,
    timeout: Duration,
}
impl Requests {
    pub fn new(policy: &crate::config::Lsp) -> Self {
        Self {
            pending: BTreeMap::new(),
            next: 3,
            limit: policy.max_pending_requests,
            timeout: Duration::from_millis(policy.request_timeout_ms as u64),
        }
    }
    pub fn deadline(&self) -> Option<Instant> {
        self.pending.values().map(|p| p.deadline).min()
    }
    pub fn sweep(&mut self, output: &Outbox) -> Result<(), String> {
        let obsolete: Vec<_> = self
            .pending
            .iter()
            .filter(|(_, p)| !p.query.current() || Instant::now() >= p.deadline)
            .map(|(id, _)| *id)
            .collect();
        for id in obsolete {
            let pending = self.pending.remove(&id).expect("obsolete LSP request");
            let reason = if pending.query.current() {
                Fallback::Timeout
            } else {
                Fallback::Cancelled
            };
            pending.query.ticket.finish(Reply::fallback(reason));
            cancel(id, output)?;
        }
        Ok(())
    }
    pub fn start(
        &mut self,
        query: Query,
        permit: Permit,
        wire: (&Outbox, &Capabilities),
    ) -> Result<(), String> {
        self.replace(query.stamp.buffer, wire.0)?;
        if !query.current() {
            return Ok(());
        }
        if !wire.1.completion {
            query.ticket.finish(Reply::fallback(Fallback::Unsupported));
            return Ok(());
        }
        if self.pending.len() >= self.limit {
            return Ok(());
        }
        let id = self.next;
        self.next = self
            .next
            .checked_add(1)
            .ok_or("LSP request identity exhausted")?;
        let position = position::encode(&query.text, query.point, &wire.1.encoding)?;
        let context = context(&query, wire.1);
        wire.0.document(
            &json!({"jsonrpc":"2.0","id":id,"method":"textDocument/completion","params":{
                "textDocument":{"uri":query.uri},"position":position,"context":context
            }}),
            permit,
        )?;
        self.pending.insert(
            id,
            Pending {
                query,
                deadline: Instant::now() + self.timeout,
                encoding: wire.1.encoding.clone(),
            },
        );
        Ok(())
    }
    fn replace(&mut self, buffer: crate::buffer::BufferId, output: &Outbox) -> Result<(), String> {
        let id = self
            .pending
            .iter()
            .find(|(_, p)| p.query.stamp.buffer == buffer)
            .map(|(id, _)| *id);
        if let Some(id) = id {
            let old = self.pending.remove(&id).expect("previous LSP request");
            old.query
                .ticket
                .finish(Reply::fallback(Fallback::Cancelled));
            cancel(id, output)?;
        }
        Ok(())
    }
    pub fn response(&mut self, message: Message) {
        let Some(id) = message.id.and_then(|id| id.as_u64()) else {
            return;
        };
        let Some(pending) = self.pending.remove(&id) else {
            return;
        };
        if !pending.query.current() {
            return;
        }
        if message.error.is_some() {
            pending
                .query
                .ticket
                .finish(Reply::fallback(Fallback::Unavailable));
        } else {
            pending
                .query
                .finish(message.result.unwrap_or(Value::Null), &pending.encoding);
        }
    }
}
fn cancel(id: u64, output: &Outbox) -> Result<(), String> {
    output.send(&json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":id}}))
}
fn context(query: &Query, caps: &Capabilities) -> Value {
    if query.manual {
        return json!({"triggerKind":1});
    }
    let previous = query
        .point
        .checked_sub(1)
        .map(|at| query.text.char(at).to_string());
    if let Some(previous) = previous.filter(|ch| caps.triggers.contains(ch)) {
        return json!({"triggerKind":2,"triggerCharacter":previous});
    }
    json!({"triggerKind":if query.token.incomplete.load(Ordering::Acquire) { 3 } else { 1 }})
}
