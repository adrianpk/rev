use std::io;

use crate::file::{Rename, Renamed};

use super::Tree;

impl Tree {
    pub fn prepare_rename(&self) -> io::Result<Rename> {
        let entry = self
            .selected()
            .ok_or_else(|| io::Error::other("no tree selection"))?;
        Rename::capture(entry.path())
    }

    /// Rebase only loaded paths, then refresh the same expanded branches.
    /// This follows a renamed root and keeps selection inside a moved subtree.
    pub fn renamed(&mut self, renamed: &Renamed) -> io::Result<()> {
        for entry in &mut self.entries {
            if let Ok(suffix) = entry.path.strip_prefix(renamed.source()) {
                entry.path = if suffix.as_os_str().is_empty() {
                    renamed.target().into()
                } else {
                    renamed.target().join(suffix)
                };
            }
        }
        if let Some(root) = &self.root
            && let Ok(suffix) = root.strip_prefix(renamed.source())
        {
            self.root = Some(if suffix.as_os_str().is_empty() {
                renamed.target().into()
            } else {
                renamed.target().join(suffix)
            });
        }
        self.refresh()
    }
}
