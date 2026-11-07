use std::path::Path;

use crate::window::ViewId;

use super::{Entry, Tree};

impl Tree {
    /// Resolve only on an editing identity/focus transition or a published
    /// listing. Drawing and same-file point movement consume retained state.
    pub(crate) fn follow(&mut self, view: ViewId, path: Option<&Path>) {
        if self.active_view == Some(view)
            && self.active_path.as_deref() == path
            && !self.follow_dirty
        {
            return;
        }
        self.active_view = Some(view);
        if self.active_path.as_deref() != path {
            self.active_path = path.map(Path::to_path_buf);
        }
        self.follow_dirty = false;
        if !self.policy.follow_active_buffer {
            return;
        }
        let matches = |entry: &Entry| {
            !entry.directory
                && !entry.unavailable
                && path.is_some_and(|path| {
                    if entry.symlink {
                        entry.source.as_deref() == Some(path)
                    } else {
                        entry.path == path
                    }
                })
        };
        self.active = self
            .selected()
            .filter(|entry| matches(entry))
            .map(|_| self.selected)
            .or_else(|| self.entries.iter().position(matches));
        self.follow_pending = true;
    }

    /// Successful ingress establishes an alias association without probing it
    /// again. A refreshed entry discards that evidence until another visit.
    pub(crate) fn admit_file(&mut self, path: &Path, source: &Path) {
        for entry in &mut self.entries {
            if entry.symlink && entry.path == path {
                entry.source = Some(source.to_path_buf());
                self.follow_dirty = true;
            }
        }
    }

    pub(crate) fn row_selected(&self, row: u16) -> bool {
        let selected = if self.focused || !self.policy.follow_active_buffer {
            self.entries.get(self.selected).map(|_| self.selected)
        } else {
            self.active
        };
        selected == Some(self.top + usize::from(row))
    }
}
