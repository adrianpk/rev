//! Optional session-owned language server transport. Disabled clients create no
//! threads or processes. IO, JSON work and deadlines belong to connection workers.
mod attachment;
mod budget;
pub(crate) mod completion;
mod connection;
mod documents;
mod frame;
mod inbox;
mod io;
pub mod position;
mod protocol;
mod requests;
pub mod roots;

use crate::config::Lsp;
use connection::Connection;
pub use documents::DocumentVersion;
pub use protocol::{Capabilities, Encoding, Save};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Disabled,
    Excluded,
    Unavailable,
    Starting,
    Ready(Capabilities),
    Missing(String),
    Failed(String),
    Limited,
    Stopping,
}

/// Connection identities use a frozen server entry plus a canonical root.
/// Failed entries remain retained until explicit disable/enable.
pub struct Client {
    policy: Lsp,
    connections: BTreeMap<(String, PathBuf), Connection>,
    selected: Option<(String, PathBuf)>,
    selected_buffer: Option<crate::buffer::BufferId>,
    fallback: Status,
    attachments: BTreeMap<crate::buffer::BufferId, attachment::Attachment>,
    excluded: std::collections::BTreeSet<crate::buffer::BufferId>,
    snapshot_bytes: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    next_connection: u64,
    next_document: u64,
}

impl Client {
    pub fn new(policy: Lsp) -> Self {
        Self {
            policy,
            connections: BTreeMap::new(),
            selected: None,
            selected_buffer: None,
            fallback: Status::Unavailable,
            attachments: BTreeMap::new(),
            excluded: std::collections::BTreeSet::new(),
            snapshot_bytes: std::sync::Arc::default(),
            next_connection: 1,
            next_document: 1,
        }
    }

    pub fn enabled(&self) -> bool {
        self.policy.enabled
    }

    /// Disabling requests shutdown without waiting on the terminal thread.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.policy.enabled = enabled;
        if !enabled {
            for attachment in self.attachments.values() {
                attachment.token.retire();
            }
            self.attachments.clear();
            for connection in self.connections.values_mut() {
                connection.retiring = true;
                connection.inbox.stop();
            }
        }
        self.collect();
    }

    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.fallback = Status::Unavailable;
    }

    pub fn activate(&mut self, file: &Path, language: &str, tree: Option<&Path>) -> Status {
        self.collect();
        self.selected = None;
        let result = self.attach(file, language, tree);
        self.fallback = result.clone();
        result
    }

    fn attach(&mut self, file: &Path, language: &str, tree: Option<&Path>) -> Status {
        if !self.enabled() {
            return Status::Disabled;
        }
        let Some((name, server)) = self
            .policy
            .servers
            .iter()
            .find(|(_, server)| server.languages.iter().any(|id| id == language))
        else {
            return Status::Unavailable;
        };
        let root = match roots::resolve(file, tree, &server.root_markers) {
            Ok(root) => root,
            Err(error) => return Status::Failed(error),
        };
        let key = (name.clone(), root.clone());
        self.selected = Some(key.clone());
        if let Some(connection) = self.connections.get(&key) {
            return status(connection);
        }
        if self.connections.len() >= self.policy.max_servers {
            return Status::Limited;
        }
        let generation = self.next_connection;
        let Some(next) = generation.checked_add(1) else {
            return Status::Limited;
        };
        self.next_connection = next;
        match Connection::start(server.clone(), root, self.policy.clone(), generation) {
            Ok(connection) => {
                self.connections.insert(key, connection);
                Status::Starting
            }
            Err(error) => Status::Failed(error),
        }
    }

    /// Retrieve current status without IO, decoding or another launch attempt.
    pub fn status(&self) -> Status {
        if !self.enabled() {
            return Status::Disabled;
        }
        self.selected
            .as_ref()
            .and_then(|key| self.connections.get(key))
            .map(status)
            .unwrap_or_else(|| self.fallback.clone())
    }

    /// Retained stderr is diagnostic data and never protocol input.
    pub fn stderr_bytes(&self) -> usize {
        self.connections
            .values()
            .map(|connection| connection.stderr.lock().expect("LSP stderr lock").len())
            .sum()
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    pub fn starting(&self) -> bool {
        self.connections.values().any(|connection| {
            connection.retiring || matches!(status(connection), Status::Starting | Status::Stopping)
        })
    }

    pub fn collect(&mut self) -> bool {
        let before = self.connections.len();
        let disabled = !self.policy.enabled;
        self.connections.retain(|_, connection| {
            if connection.finished() && (disabled || connection.retiring) {
                connection.join();
                false
            } else {
                true
            }
        });
        before != self.connections.len()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // Notify every worker before joining any one deadline.
        for connection in self.connections.values() {
            connection.inbox.stop();
        }
    }
}

fn status(connection: &Connection) -> Status {
    connection.status.lock().expect("LSP status lock").clone()
}
