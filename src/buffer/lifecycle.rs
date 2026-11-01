use super::{Buffer, BufferId, Buffers, Error, Stamp};

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
        let fresh = if self.entries.len() == 1 {
            Some(Buffer::new("", &self.settings.settings().files)?)
        } else {
            None
        };
        let replacement = fresh
            .as_ref()
            .map(Buffer::id)
            .or_else(|| self.entries.keys().copied().find(|id| *id != kill.id()))
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
