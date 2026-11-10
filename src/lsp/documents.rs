use super::{Capabilities, budget::Permit, inbox::Inbox, io::Outbox, position, protocol::Save};
use crate::buffer::{
    BufferId, Stamp,
    edit::{Observer, Published},
};
use ropey::Rope;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
};

pub(super) const DOCUMENT_BYTES: usize = 8 * 1024 * 1024;
const SNAPSHOT_BYTES: usize = 32 * 1024 * 1024;

/// Synchronization evidence is qualified by all identities, even after reopening
/// a document whose bytes happen to equal an earlier revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentVersion {
    pub connection: u64,
    pub generation: u64,
    pub version: i32,
    pub stamp: Stamp,
}

pub(super) struct Activation {
    pub id: BufferId,
    pub connection: u64,
    pub generation: u64,
    state: AtomicU8,
    bytes: Mutex<usize>,
    total: Arc<AtomicUsize>,
    version: Mutex<Option<DocumentVersion>>,
    pub incomplete: AtomicBool,
}

impl Activation {
    pub fn new(
        stamp: Stamp,
        generation: (u64, u64),
        total: Arc<AtomicUsize>,
        bytes: usize,
    ) -> Result<Arc<Self>, String> {
        let result = Arc::new(Self {
            id: stamp.buffer,
            connection: generation.0,
            generation: generation.1,
            state: AtomicU8::new(0),
            bytes: Mutex::new(0),
            total,
            version: Mutex::new(None),
            incomplete: AtomicBool::new(false),
        });
        if !result.resize(bytes) {
            return Err("LSP document snapshot limit exceeded".into());
        }
        Ok(result)
    }

    pub fn active(&self) -> bool {
        self.state.load(Ordering::Acquire) == 0
    }
    pub fn limited(&self) -> bool {
        self.state.load(Ordering::Acquire) == 1
    }
    pub fn retire(&self) {
        self.state.store(2, Ordering::Release);
        self.release();
    }
    pub fn limit(&self) {
        self.state.store(1, Ordering::Release);
        self.release();
    }

    pub fn resize(&self, bytes: usize) -> bool {
        if bytes > DOCUMENT_BYTES {
            return false;
        }
        let mut retained = self.bytes.lock().expect("LSP snapshot lock");
        if !self.active() {
            return false;
        }
        let old = *retained;
        let admitted = self
            .total
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |total| {
                (total - old)
                    .checked_add(bytes)
                    .filter(|total| *total <= SNAPSHOT_BYTES)
            })
            .is_ok();
        if admitted {
            *retained = bytes;
        }
        admitted
    }

    fn release(&self) {
        let bytes = std::mem::take(&mut *self.bytes.lock().expect("LSP snapshot lock"));
        self.total.fetch_sub(bytes, Ordering::AcqRel);
        *self.version.lock().expect("LSP version lock") = None;
    }

    pub fn version(&self) -> Option<DocumentVersion> {
        if !self.active() {
            return None;
        }
        *self.version.lock().expect("LSP version lock")
    }

    fn synchronized(&self, stamp: Stamp, version: i32) {
        let mut current = self.version.lock().expect("LSP version lock");
        if !self.active() {
            return;
        }
        *current = Some(DocumentVersion {
            connection: self.connection,
            generation: self.generation,
            version,
            stamp,
        });
    }
}

impl Drop for Activation {
    fn drop(&mut self) {
        self.release();
    }
}

pub(super) struct Sink {
    pub activation: Arc<Activation>,
    pub inbox: Arc<Inbox>,
}
impl Observer for Sink {
    fn active(&self) -> bool {
        self.activation.active() && !self.inbox.closed()
    }
    fn published(&self, publication: Published) {
        if !self.active() {
            return;
        }
        if !self.activation.resize(publication.text.len_bytes()) {
            self.activation.limit();
            self.inbox
                .document(Event::Close(self.activation.id, self.activation.generation));
            return;
        }
        self.inbox
            .document(Event::Change(publication, Arc::clone(&self.activation)));
    }
}

pub(super) enum Event {
    Open {
        token: Arc<Activation>,
        stamp: Stamp,
        uri: String,
        language: String,
        text: Rope,
    },
    Change(Published, Arc<Activation>),
    Close(BufferId, u64),
    Save {
        token: Arc<Activation>,
        stamp: Stamp,
        text: Rope,
    },
}
impl Event {
    pub fn retire(&self) {
        if let Some(token) = self.token() {
            token.retire();
        }
    }
    fn token(&self) -> Option<&Arc<Activation>> {
        match self {
            Self::Open { token, .. } | Self::Change(_, token) | Self::Save { token, .. } => {
                Some(token)
            }
            Self::Close(..) => None,
        }
    }
    pub fn bytes(&self) -> usize {
        match self {
            Self::Open {
                uri,
                language,
                text,
                ..
            } => uri.len() + language.len() + text.len_bytes(),
            Self::Change(value, _) => value
                .original
                .len_bytes()
                .saturating_add(value.text.len_bytes())
                .saturating_add(
                    value
                        .changes
                        .len()
                        .saturating_mul(std::mem::size_of::<crate::buffer::edit::Change>()),
                ),
            Self::Close(..) => 0,
            Self::Save { text, .. } => text.len_bytes(),
        }
    }
}

struct Opened {
    token: Arc<Activation>,
    uri: String,
    generation: u64,
    stamp: Stamp,
    version: i32,
}
#[derive(Default)]
pub(super) struct Documents(BTreeMap<BufferId, Opened>);
impl Drop for Documents {
    fn drop(&mut self) {
        for doc in self.0.values() {
            doc.token.retire();
        }
    }
}
impl Documents {
    pub fn process(
        &mut self,
        event: Event,
        permit: Permit,
        output: &Outbox,
        caps: &Capabilities,
    ) -> Result<(), String> {
        let token = event.token().cloned();
        let result = self.process_event(event, permit, output, caps);
        if result.is_err()
            && let Some(token) = token
        {
            token.retire();
        }
        result
    }

    fn process_event(
        &mut self,
        event: Event,
        permit: Permit,
        output: &Outbox,
        caps: &Capabilities,
    ) -> Result<(), String> {
        match event {
            Event::Open {
                token,
                stamp,
                uri,
                language,
                text,
            } => self.open(
                Open {
                    token,
                    stamp,
                    uri,
                    language,
                    text,
                },
                permit,
                output,
                caps,
            ),
            Event::Change(value, token) => self.change(&value, &token, permit, (output, caps)),
            Event::Close(id, generation) => self.close((id, generation), permit, output, caps),
            Event::Save { token, stamp, text } => {
                self.save(&Saved { token, stamp, text }, permit, output, caps)
            }
        }
    }

    fn open(
        &mut self,
        value: Open,
        permit: Permit,
        output: &Outbox,
        caps: &Capabilities,
    ) -> Result<(), String> {
        if !value.token.active() {
            return Ok(());
        }
        if self.0.contains_key(&value.stamp.buffer) {
            return Err("LSP document opened twice without close".into());
        }
        if caps.open_close {
            output.document(&json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{
                "uri":value.uri,"languageId":value.language,"version":1,"text":value.text.to_string()
            }}}), permit)?;
        }
        value.token.synchronized(value.stamp, 1);
        self.0.insert(
            value.stamp.buffer,
            Opened {
                uri: value.uri,
                generation: value.token.generation,
                token: value.token,
                stamp: value.stamp,
                version: 1,
            },
        );
        Ok(())
    }

    fn change(
        &mut self,
        value: &Published,
        token: &Activation,
        permit: Permit,
        wire: (&Outbox, &Capabilities),
    ) -> Result<(), String> {
        if !token.active() {
            return Ok(());
        }
        let doc = self.current(value.before, token)?;
        let version = doc
            .version
            .checked_add(1)
            .ok_or("LSP document version exhausted")?;
        let changes = changes(value, wire.1)?;
        wire.0.document(
            &json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
                "textDocument":{"uri":doc.uri,"version":version},"contentChanges":changes
            }}),
            permit,
        )?;
        doc.stamp = value.after;
        doc.version = version;
        token.synchronized(value.after, version);
        Ok(())
    }

    fn current(&mut self, stamp: Stamp, token: &Activation) -> Result<&mut Opened, String> {
        let doc = self
            .0
            .get_mut(&stamp.buffer)
            .ok_or("LSP change precedes document open")?;
        if doc.generation != token.generation || doc.stamp != stamp {
            return Err("LSP document generation or revision is obsolete".into());
        }
        Ok(doc)
    }

    fn close(
        &mut self,
        identity: (BufferId, u64),
        permit: Permit,
        output: &Outbox,
        caps: &Capabilities,
    ) -> Result<(), String> {
        if !self
            .0
            .get(&identity.0)
            .is_some_and(|doc| doc.generation == identity.1)
        {
            return Ok(());
        }
        let doc = self.0.remove(&identity.0).expect("checked LSP document");
        if !caps.open_close {
            return Ok(());
        }
        output.document(&json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":doc.uri}}}), permit)
    }

    fn save(
        &mut self,
        saved: &Saved,
        permit: Permit,
        output: &Outbox,
        caps: &Capabilities,
    ) -> Result<(), String> {
        if !saved.token.active() || caps.save == Save::None {
            return Ok(());
        }
        let doc = self.current(saved.stamp, &saved.token)?;
        let mut params = json!({"textDocument":{"uri":doc.uri}});
        if caps.save == Save::Text {
            params["text"] = json!(saved.text.to_string());
        }
        output.document(
            &json!({"jsonrpc":"2.0","method":"textDocument/didSave","params":params}),
            permit,
        )
    }
}

struct Open {
    token: Arc<Activation>,
    stamp: Stamp,
    uri: String,
    language: String,
    text: Rope,
}
struct Saved {
    token: Arc<Activation>,
    stamp: Stamp,
    text: Rope,
}

fn changes(value: &Published, caps: &Capabilities) -> Result<Vec<Value>, String> {
    if caps.sync == 1 {
        return Ok(vec![json!({"text":value.text.to_string()})]);
    }
    value
        .changes
        .iter()
        .rev()
        .map(|change| {
            let range = change.before();
            let start = position::encode(&value.original, range.start, &caps.encoding)?;
            let end = position::encode(&value.original, range.end, &caps.encoding)?;
            Ok(json!({"range":{"start":start,"end":end},"text":change.inserted().to_string()}))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    // Aggregate admission and explicit retirement share one ledger; a failed
    // resize cannot consume capacity and repeated cleanup cannot release twice.
    #[test]
    fn snapshot_accounting() {
        let mut buffers = crate::buffer::Buffers::new().unwrap();
        let total = Arc::new(AtomicUsize::new(0));
        let mut tokens = Vec::new();
        for generation in 1..=4 {
            let id = buffers.create("").unwrap();
            tokens.push(
                Activation::new(
                    buffers.get(id).unwrap().stamp(),
                    (1, generation),
                    Arc::clone(&total),
                    DOCUMENT_BYTES,
                )
                .unwrap(),
            );
        }
        let stamp = buffers.get(buffers.initial()).unwrap().stamp();
        assert!(Activation::new(stamp, (1, 5), Arc::clone(&total), DOCUMENT_BYTES).is_err());
        assert_eq!(total.load(Ordering::Acquire), SNAPSHOT_BYTES);
        tokens[0].retire();
        let restored = Activation::new(stamp, (1, 6), Arc::clone(&total), DOCUMENT_BYTES).unwrap();
        assert!(!tokens[1].resize(DOCUMENT_BYTES + 1));
        assert_eq!(total.load(Ordering::Acquire), SNAPSHOT_BYTES);
        tokens[2].limit();
        assert!(tokens[3].resize(DOCUMENT_BYTES / 2));
        drop(tokens);
        drop(restored);
        assert_eq!(total.load(Ordering::Acquire), 0);
    }
    // Failed initialization must release queued snapshots; an initialized worker
    // must release its retained lease even while the UI still holds the token.
    #[test]
    fn failed_worker_leases() {
        let buffers = crate::buffer::Buffers::new().unwrap();
        let stamp = buffers.get(buffers.initial()).unwrap().stamp();
        let total = Arc::new(AtomicUsize::new(0));
        let token = Activation::new(stamp, (1, 1), Arc::clone(&total), 16).unwrap();
        let inbox = Inbox::default();
        inbox.document(Event::Open {
            token: Arc::clone(&token),
            stamp,
            uri: "file:///owned".into(),
            language: "rust".into(),
            text: Rope::from_str("snapshot"),
        });
        inbox.close();
        assert!(!token.active());
        assert_eq!(total.load(Ordering::Acquire), 0);
        let token = Activation::new(stamp, (1, 2), Arc::clone(&total), 16).unwrap();
        let mut documents = Documents::default();
        token.synchronized(stamp, 1);
        documents.0.insert(
            stamp.buffer,
            Opened {
                token: Arc::clone(&token),
                uri: "file:///owned".into(),
                generation: 2,
                stamp,
                version: 1,
            },
        );
        drop(documents);
        assert!(!token.active());
        assert!(token.version().is_none());
        assert!(!token.resize(1));
        assert_eq!(total.load(Ordering::Acquire), 0);
    }

    // Retirement races with canonical publication. Both operations must share
    // the byte lease so an interrupted resize cannot leak or subtract twice.
    #[test]
    fn concurrent_retirement() {
        let buffers = crate::buffer::Buffers::new().unwrap();
        let stamp = buffers.get(buffers.initial()).unwrap().stamp();
        let total = Arc::new(AtomicUsize::new(0));
        for generation in 1..=100 {
            let token = Activation::new(stamp, (1, generation), Arc::clone(&total), 16).unwrap();
            let publisher = Arc::clone(&token);
            let worker = std::thread::spawn(move || {
                for _ in 0..100 {
                    publisher.resize(32);
                    publisher.synchronized(stamp, 2);
                }
            });
            token.retire();
            worker.join().unwrap();
            assert_eq!(total.load(Ordering::Acquire), 0);
            assert!(token.version().is_none());
        }
    }
}
