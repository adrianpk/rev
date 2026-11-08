use std::io;

use super::Tree;

impl Tree {
    /// Scroll only the loaded preorder; no expansion, directory read or focus
    /// transition. Keep selection unless the visible interval excludes it.
    pub(crate) fn scroll_viewport(&mut self, backward: bool, distance: usize, rows: u16) -> bool {
        if rows == 0 {
            return false;
        }
        // Explicit wheel intent supersedes a follow reveal awaiting its frame.
        self.follow_pending = false;
        let before = (self.top, self.selected);
        let last = self.entries.len().saturating_sub(1);
        self.top = if backward {
            self.top.saturating_sub(distance)
        } else {
            self.top.saturating_add(distance).min(last)
        };
        let bottom = self.top.saturating_add(usize::from(rows) - 1).min(last);
        self.selected = self.selected.clamp(self.top, bottom);
        (self.top, self.selected) != before
    }

    /// Move only within the loaded visible preorder; endpoints do not wrap.
    pub fn move_selection(&mut self, forward: bool) {
        self.move_selection_by(forward, 1);
    }

    /// Counted selection uses only the already loaded preorder, including the
    /// parent row. It does not expand, refresh or visit any path.
    pub(crate) fn move_selection_by(&mut self, forward: bool, distance: usize) {
        self.selected = if forward {
            self.selected
                .saturating_add(distance)
                .min(self.entries.len().saturating_sub(1))
        } else {
            self.selected.saturating_sub(distance)
        };
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
        let children = match entry.children(&self.policy) {
            Ok(children) => children,
            Err(error) => {
                if entry.depth == 0 && entry.root_id.is_some() {
                    self.entries[self.selected].unavailable = true;
                }
                return Err(error);
            }
        };
        self.entries[self.selected].unavailable = false;
        self.entries[self.selected].expanded = true;
        self.entries[self.selected].incomplete = children.incomplete;
        if self.selected == 0 {
            self.initialized = true;
        }
        let start = self.children_start(self.selected);
        self.entries.splice(start..start, children.entries);
        self.follow_dirty = true;
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
            let parent = self
                .entries
                .get(self.selected + 1)
                .filter(|entry| entry.parent)
                .cloned();
            self.entries.drain(self.selected + 1..end);
            if let Some(parent) = parent {
                self.entries.insert(self.selected + 1, parent);
            }
            self.follow_dirty = true;
            self.entries[self.selected].expanded = false;
            self.entries[self.selected].incomplete = false;
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
        if entry.parent {
            return self.up_directory();
        }
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
