use super::{Buffer, BufferId, Buffers, Error, Stamp};

impl Buffer {
    /// Private read-only projection: no registry, file, observer or edit history.
    pub(crate) fn terminal_copy(
        text: &str,
        settings: &crate::config::Snapshot,
    ) -> Result<Self, Error> {
        let mut buffer = Self::new(text, &settings.settings().files)?;
        buffer.name = "Terminal copy".into();
        buffer.transient = true;
        buffer.read_only = true;
        buffer.mode_choice = Some(super::Mode::Text);
        buffer.apply_settings(settings);
        buffer.wrap_override = Some(false);
        buffer.org_enabled = false;
        Ok(buffer)
    }
}

/// A close decision identifies one live buffer and its exact text revision.
/// Newer edits invalidate discard or save-before-close authority.
pub struct Kill {
    stamp: Stamp,
}

impl Kill {
    pub fn id(&self) -> BufferId {
        self.stamp.buffer
    }
}

impl Buffers {
    /// Retire all source references before a checked context replacement.
    /// Work on a bounded copy so an invalid bookmark leaves the owner unchanged.
    pub(crate) fn context_bookmarks(&self) -> Result<crate::bookmark::Bookmarks, Error> {
        let mut bookmarks = self
            .bookmarks
            .lock()
            .expect("bookmark owner poisoned")
            .clone();
        for buffer in self.iter().filter(|buffer| buffer.terminal().is_none()) {
            bookmarks.checkpoint(buffer, true)?;
        }
        Ok(bookmarks)
    }
    pub(crate) fn inherit_bookmarks(&mut self, owner: crate::bookmark::Shared) {
        self.bookmarks = owner;
        for buffer in self.entries.values_mut() {
            buffer.bookmarks = self.bookmarks.clone();
        }
    }
    pub(crate) fn terminal(&mut self, id: crate::term::SessionId) -> Result<BufferId, Error> {
        let mut buffer = Buffer::new("", &self.settings.settings().files)?;
        buffer.name = if id.number() == 1 {
            "terminal".into()
        } else {
            format!("terminal-{}", id.number())
        };
        buffer.resource = super::Resource::Terminal(id);
        buffer.transient = true;
        buffer.mode_choice = Some(super::Mode::Text);
        Ok(self.insert(buffer))
    }
    pub(crate) fn rename_terminal(&mut self, id: BufferId, name: &str) -> Result<(), String> {
        if name.trim().is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
            return Err("Terminal name must contain 1-256 bytes without control characters".into());
        }
        if self.iter().any(|b| b.id() != id && b.name() == name) {
            return Err("A buffer already has that name".into());
        }
        let buffer = self.get_mut(id).map_err(|e| e.to_string())?;
        if buffer.terminal().is_none() {
            return Err("Rename requires a terminal buffer".into());
        }
        buffer.name = name.into();
        Ok(())
    }
    pub(crate) fn apply_ai_definition(&mut self, name: &str, incoming: &crate::config::Snapshot) {
        self.settings.ai_definition(name, incoming);
    }

    /// A result uses the canonical buffer/view machinery, with no file or undo
    /// publication. Names retain the existing unique-label policy.
    pub(crate) fn result(&mut self, name: &str, text: &str) -> Result<BufferId, Error> {
        self.result_mode(name, text, super::Mode::Text)
    }

    pub(crate) fn org_result(&mut self, name: &str, text: &str) -> Result<BufferId, Error> {
        self.result_mode(name, text, super::Mode::Org)
    }

    fn result_mode(
        &mut self,
        name: &str,
        text: &str,
        mode: super::Mode,
    ) -> Result<BufferId, Error> {
        let mut buffer = Buffer::new(text, &self.settings.settings().files)?;
        buffer.name = name.into();
        buffer.transient = true;
        buffer.mode_choice = Some(mode);
        buffer.set_readonly(true);
        Ok(self.insert(buffer))
    }
    pub(crate) fn apply_lsp_settings(&mut self, incoming: &crate::config::Snapshot) {
        self.settings.lsp_delta(incoming);
        for buffer in self.entries.values_mut() {
            buffer.apply_settings(&self.settings);
        }
    }
    /// Inventory borrows live text owners in stable identity order.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Buffer> + ExactSizeIterator {
        self.entries.values()
    }

    pub fn named(&self, name: &str) -> Option<BufferId> {
        self.iter()
            .find(|buffer| buffer.name() == name)
            .map(Buffer::id)
    }

    pub fn prepare_kill(&self, id: BufferId) -> Result<Kill, Error> {
        Ok(Kill {
            stamp: self.get(id)?.stamp(),
        })
    }

    /// Consume explicit discard authority or a completed save's close request.
    /// Allocate a final-buffer replacement before removing anything. The caller
    /// replaces every view reference with the returned deterministic live owner.
    pub fn kill(&mut self, kill: Kill) -> Result<BufferId, Error> {
        let fresh = if self
            .entries
            .values()
            .all(|b| b.id() == kill.id() || b.terminal().is_some())
        {
            Some(Buffer::new("", &self.settings.settings().files)?)
        } else {
            None
        };
        let replacement = fresh
            .as_ref()
            .map(Buffer::id)
            .or_else(|| {
                self.entries
                    .values()
                    .find(|b| b.id() != kill.id() && b.terminal().is_none())
                    .map(Buffer::id)
            })
            .ok_or(Error::Missing(kill.id()))?;
        self.checkpoint_close(kill.stamp)?;
        let removed = self
            .entries
            .remove(&kill.id())
            .ok_or(Error::Missing(kill.id()))?;
        if let Some(file) = removed.file {
            self.files.remove(file.path());
        }
        self.replace_killed(kill.id(), replacement, fresh);
        Ok(replacement)
    }
    fn checkpoint_close(&self, stamp: Stamp) -> Result<(), Error> {
        let buffer = self.get(stamp.buffer)?;
        buffer.current(stamp)?;
        if buffer.terminal().is_some() {
            return Ok(());
        }
        let closed = self
            .locations
            .lock()
            .expect("location owner poisoned")
            .prepare_close(buffer)?;
        self.bookmarks
            .lock()
            .expect("bookmark owner poisoned")
            .checkpoint(buffer, true)?;
        self.locations
            .lock()
            .expect("location owner poisoned")
            .publish_close(closed);
        Ok(())
    }

    fn replace_killed(&mut self, id: BufferId, replacement: BufferId, fresh: Option<Buffer>) {
        if let Some(mut buffer) = fresh {
            buffer.bookmarks = self.bookmarks.clone();
            buffer.locations = self.locations.clone();
            buffer.name = "*scratch*".into();
            buffer.apply_settings(&self.settings);
            self.entries.insert(buffer.id(), buffer);
        }
        if self.initial == id {
            self.initial = replacement;
        }
    }
}

/// Every fallible close admission, including the last-buffer replacement, is
/// staged before a Workspace membership publication can become durable.
pub(crate) struct Closing {
    stamps: Vec<Stamp>,
    locations: crate::location::Closing,
    bookmarks: crate::bookmark::Bookmarks,
    scratch: Option<Buffer>,
    replacement: BufferId,
}
impl Buffers {
    pub(crate) fn prepare_closing(&self, ids: &[BufferId]) -> Result<Closing, Error> {
        let mut bookmarks = self
            .bookmarks
            .lock()
            .expect("bookmark owner poisoned")
            .clone();
        let mut locations = Vec::new();
        let mut stamps = Vec::new();
        for id in ids {
            let buffer = self.get(*id)?;
            bookmarks.checkpoint(buffer, true)?;
            locations.extend(
                self.locations
                    .lock()
                    .expect("location owner poisoned")
                    .prepare_close(buffer)?,
            );
            stamps.push(buffer.stamp());
        }
        let retained = self
            .iter()
            .find(|b| !ids.contains(&b.id()) && b.terminal().is_none())
            .map(Buffer::id);
        let scratch = if retained.is_none() {
            Some(Buffer::new("", &self.settings.settings().files)?)
        } else {
            None
        };
        let replacement = retained
            .or_else(|| scratch.as_ref().map(Buffer::id))
            .ok_or(Error::Exhausted)?;
        Ok(Closing {
            stamps,
            locations,
            bookmarks,
            scratch,
            replacement,
        })
    }
    pub(crate) fn publish_closing(
        &mut self,
        closing: Closing,
    ) -> Result<(Vec<BufferId>, BufferId), Error> {
        for stamp in &closing.stamps {
            self.get(stamp.buffer)?.current(*stamp)?;
        }
        if let Some(mut buffer) = closing.scratch {
            buffer.name = "*scratch*".into();
            self.insert(buffer);
        }
        let mut closed = Vec::new();
        for stamp in closing.stamps {
            let buffer = self
                .entries
                .remove(&stamp.buffer)
                .expect("admitted close retains source");
            if let Some(file) = buffer.file {
                self.files.remove(file.path());
            }
            if self.initial == stamp.buffer {
                self.initial = closing.replacement;
            }
            closed.push(stamp.buffer);
        }
        *self.bookmarks.lock().expect("bookmark owner poisoned") = closing.bookmarks;
        self.locations
            .lock()
            .expect("location owner poisoned")
            .publish_close(closing.locations);
        Ok((closed, closing.replacement))
    }
}
