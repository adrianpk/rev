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
    /// Inventory borrows canonical owners in stable identity order.
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
        self.get(kill.id())?.current(kill.stamp)?;
        let fresh = if self.entries.len() == 1 {
            Some(Buffer::new("")?)
        } else {
            None
        };
        let replacement = fresh
            .as_ref()
            .map(Buffer::id)
            .or_else(|| self.entries.keys().copied().find(|id| *id != kill.id()))
            .ok_or(Error::Missing(kill.id()))?;
        let removed = self
            .entries
            .remove(&kill.id())
            .ok_or(Error::Missing(kill.id()))?;
        if let Some(file) = removed.file {
            self.files.remove(file.path());
        }
        if let Some(mut buffer) = fresh {
            buffer.name = "*scratch*".into();
            buffer.apply_settings(&self.settings);
            self.entries.insert(buffer.id(), buffer);
        }
        if self.initial == kill.id() {
            self.initial = replacement;
        }
        Ok(replacement)
    }
}
