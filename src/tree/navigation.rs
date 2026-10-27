use std::io;

use super::Tree;

impl Tree {
    /// Move only within the loaded visible preorder; endpoints do not wrap.
    pub fn move_selection(&mut self, forward: bool) {
        if forward {
            self.selected = self
                .selected
                .saturating_add(1)
                .min(self.entries.len().saturating_sub(1));
        } else {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    /// Read one directory's direct children completely before publishing them.
    /// Already expanded directories and leaves perform no I/O here.
    pub fn expand(&mut self) -> io::Result<()> {
        let Some(entry) = self.selected() else {
            return Ok(());
        };
        if !entry.directory || entry.expanded {
            return Ok(());
        }
        let children = entry.children(&self.policy)?;
        self.entries[self.selected].expanded = true;
        self.entries
            .splice(self.selected + 1..self.selected + 1, children);
        Ok(())
    }

    /// Collapse an open entry; otherwise select its visible parent.
    pub fn collapse(&mut self) {
        let Some(entry) = self.selected() else {
            return;
        };
        let depth = entry.depth;
        if entry.expanded {
            let end = self.descendants_end(self.selected);
            self.entries.drain(self.selected + 1..end);
            self.entries[self.selected].expanded = false;
        } else if depth > 0 {
            self.selected = self.entries[..self.selected]
                .iter()
                .rposition(|entry| entry.depth < depth)
                .unwrap_or(0);
        }
    }

    /// Toggle a directory. Leaf activation leaves documents and paths intact.
    pub fn activate(&mut self) -> io::Result<()> {
        let Some(entry) = self.selected() else {
            return Ok(());
        };
        if !entry.directory {
            return Err(super::entry::failure(
                "toggle directory",
                &entry.path,
                io::Error::other("entry is a leaf"),
            ));
        }
        if entry.expanded {
            self.collapse();
            Ok(())
        } else {
            self.expand()
        }
    }

    fn descendants_end(&self, start: usize) -> usize {
        let depth = self.entries[start].depth;
        self.entries[start + 1..]
            .iter()
            .position(|entry| entry.depth <= depth)
            .map_or(self.entries.len(), |offset| start + 1 + offset)
    }
}
