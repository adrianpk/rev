use super::{
    Client, Status,
    documents::{Activation, DOCUMENT_BYTES, Event, Sink},
    roots,
};
use crate::buffer::{Buffer, BufferId};
use std::path::{Path, PathBuf};
use std::sync::Arc;

type Key = (String, PathBuf);
pub(super) struct Attachment {
    pub key: Key,
    pub path: PathBuf,
    pub language: String,
    pub token: Arc<Activation>,
    tree: Option<PathBuf>,
    inbox: Arc<super::inbox::Inbox>,
}

impl Client {
    /// Bind one canonical document, independent of how many views display it.
    /// Existing associations are checked without root IO or repeated snapshots.
    pub(crate) fn attach_buffer(&mut self, buffer: &mut Buffer, tree: Option<&Path>) -> Status {
        self.selected_buffer = Some(buffer.id());
        if !self.enabled() {
            return Status::Disabled;
        }
        if self.excluded.contains(&buffer.id()) {
            return self.select_fallback(Status::Excluded);
        }
        let Some(path) = buffer.file().map(|file| file.path().to_path_buf()) else {
            return self.select_fallback(Status::Unavailable);
        };
        let language = buffer.syntax_language().to_owned();
        if self.reuse_buffer(buffer.id(), &path, &language, tree) {
            return self.buffer_status(buffer.id());
        }
        self.detach(buffer.id());
        buffer.observe(None);
        if buffer.is_transient() {
            return self.select_fallback(Status::Unavailable);
        }
        if buffer.len_bytes() > DOCUMENT_BYTES
            || self
                .snapshot_bytes
                .load(std::sync::atomic::Ordering::Acquire)
                .saturating_add(buffer.len_bytes())
                > 32 * 1024 * 1024
        {
            return self.select_fallback(Status::Limited);
        }
        let status = self.activate(&path, &language, tree);
        if !matches!(status, Status::Starting | Status::Ready(_)) {
            return status;
        }
        match self.subscribe(buffer, path, language, tree) {
            Ok(()) => status,
            Err(error) => self.select_fallback(Status::Failed(error)),
        }
    }

    fn reuse_buffer(
        &mut self,
        id: BufferId,
        path: &Path,
        language: &str,
        tree: Option<&Path>,
    ) -> bool {
        let Some(attachment) = self.attachments.get(&id) else {
            return false;
        };
        if attachment.path != path
            || attachment.language != language
            || attachment.tree.as_deref() != tree
            || !attachment.token.active()
        {
            return false;
        }
        self.selected = Some(attachment.key.clone());
        true
    }

    fn subscribe(
        &mut self,
        buffer: &mut Buffer,
        path: PathBuf,
        language: String,
        tree: Option<&Path>,
    ) -> Result<(), String> {
        let key = self
            .selected
            .clone()
            .ok_or("LSP connection is unavailable")?;
        let connection = self
            .connections
            .get(&key)
            .ok_or("LSP connection is unavailable")?;
        let generation = self.next_document;
        self.next_document = generation
            .checked_add(1)
            .ok_or("LSP activation generation exhausted")?;
        let token = Activation::new(
            buffer.stamp(),
            (connection.generation, generation),
            self.snapshot_bytes.clone(),
            buffer.len_bytes(),
        )?;
        let uri = roots::uri(&path)?;
        let inbox = connection.inbox.clone();
        inbox.document(Event::Open {
            token: token.clone(),
            stamp: buffer.stamp(),
            uri,
            language: language.clone(),
            text: buffer.text_snapshot(),
        });
        buffer.observe(Some(Arc::new(Sink {
            activation: token.clone(),
            inbox: inbox.clone(),
        })));
        self.attachments.insert(
            buffer.id(),
            Attachment {
                key,
                path,
                language,
                token,
                inbox,
                tree: tree.map(Path::to_path_buf),
            },
        );
        Ok(())
    }

    fn select_fallback(&mut self, status: Status) -> Status {
        self.selected = None;
        self.fallback = status.clone();
        status
    }

    pub fn buffer_status(&self, id: BufferId) -> Status {
        if !self.enabled() {
            return Status::Disabled;
        }
        if self.excluded.contains(&id) {
            return Status::Excluded;
        }
        let Some(attachment) = self.attachments.get(&id) else {
            return if self.selected_buffer == Some(id) {
                self.status()
            } else {
                Status::Unavailable
            };
        };
        if attachment.token.limited() {
            return Status::Limited;
        }
        self.connections
            .get(&attachment.key)
            .map(super::status)
            .unwrap_or(Status::Unavailable)
    }

    pub fn document_version(&self, id: BufferId) -> Option<super::DocumentVersion> {
        if !self.enabled() {
            return None;
        }
        let attachment = self.attachments.get(&id)?;
        if !self
            .connections
            .get(&attachment.key)
            .is_some_and(|connection| matches!(super::status(connection), Status::Ready(_)))
        {
            return None;
        }
        attachment.token.version()
    }

    pub(crate) fn exclude(&mut self, buffer: &mut Buffer) {
        self.excluded.insert(buffer.id());
        self.detach(buffer.id());
        buffer.observe(None);
    }

    pub(crate) fn restore(&mut self, id: BufferId) {
        self.excluded.remove(&id);
    }

    pub(crate) fn closed(&mut self, id: BufferId) {
        if self.selected_buffer == Some(id) {
            self.selected_buffer = None;
        }
        self.detach(id);
        self.excluded.remove(&id);
    }

    pub(super) fn detach(&mut self, id: BufferId) {
        let Some(attachment) = self.attachments.remove(&id) else {
            return;
        };
        attachment.token.retire();
        attachment
            .inbox
            .document(Event::Close(id, attachment.token.generation));
        self.last_buffer(&attachment.key);
    }

    fn last_buffer(&mut self, key: &Key) {
        if !self.policy.shutdown_on_last_buffer
            || self
                .attachments
                .values()
                .any(|attachment| &attachment.key == key)
        {
            return;
        }
        if let Some(connection) = self.connections.get_mut(key)
            && matches!(
                super::status(connection),
                Status::Starting | Status::Ready(_)
            )
        {
            connection.retiring = true;
            connection.inbox.stop();
        }
    }

    /// Restart only the selected eligible root; exclusions and global mode survive.
    pub(crate) fn restart_buffer(&mut self, buffer: &mut Buffer, tree: Option<&Path>) {
        let status = self.attach_buffer(buffer, tree);
        if !matches!(
            status,
            Status::Starting | Status::Ready(_) | Status::Missing(_) | Status::Failed(_)
        ) {
            return;
        }
        let Some(key) = self.selected.clone() else {
            return;
        };
        self.attachments.retain(|_, attachment| {
            if attachment.key != key {
                return true;
            }
            attachment.token.retire();
            false
        });
        if let Some(connection) = self.connections.get_mut(&key) {
            connection.retiring = true;
            connection.inbox.stop();
        }
        self.collect();
    }

    pub(crate) fn saved_buffer(&mut self, buffer: &mut Buffer, tree: Option<&Path>) {
        self.attach_buffer(buffer, tree);
        if let Some(attachment) = self.attachments.get(&buffer.id()) {
            attachment.inbox.document(Event::Save {
                token: attachment.token.clone(),
                stamp: buffer.stamp(),
                text: buffer.text_snapshot(),
            });
        }
    }
}
