use std::io;
use std::path::Path;

use super::{Entry, Tree};
use crate::view::Size;

impl Tree {
    fn fixed_root(&self) -> bool {
        self.explicit_root
            || self.workspace.is_some()
            || self.policy.root == crate::config::TreeRoot::Cwd
    }

    pub(super) fn root_entries(&self, root: std::path::PathBuf) -> Vec<Entry> {
        let parent = (!self.fixed_root())
            .then(|| root.parent().filter(|path| !path.as_os_str().is_empty()))
            .flatten()
            .map(|path| Entry::parent(path.to_owned()));
        let mut entries = vec![Entry::root(root)];
        entries.extend(parent);
        entries
    }

    /// Observe an automatic root's parent before publishing any new state.
    /// Fixed roots perform no IO. The previous directory must remain a loaded
    /// directory, including when the ordinary observation limit is reached.
    pub fn up_directory(&mut self) -> io::Result<()> {
        if self.fixed_root() {
            return Err(io::Error::other("File tree root is fixed"));
        }
        let previous = self
            .root
            .as_deref()
            .ok_or_else(|| io::Error::other("File tree has no root"))?;
        let parent = previous
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or_else(|| io::Error::other("File tree root has no parent"))?;
        let mut root = Entry::root(parent.to_owned());
        let children = root.children(&self.policy)?;
        let selected = children
            .entries
            .iter()
            .position(|entry| entry.path == previous && entry.directory)
            .ok_or_else(|| {
                io::Error::other("Parent listing cannot represent the previous directory")
            })?;
        root.expanded = true;
        root.incomplete = children.incomplete;
        let mut entries = self.root_entries(parent.to_owned());
        let selected = selected + entries.len();
        entries[0] = root;
        entries.extend(children.entries);

        self.root = Some(parent.to_owned());
        self.entries = entries;
        self.selected = selected;
        self.top = 0;
        self.initialized = true;
        self.active = None;
        self.follow_dirty = true;
        self.follow_pending = false;
        self.new_width_context();
        Ok(())
    }

    /// A closed multi-root context has no implicit replacement browsing root.
    /// A later explicit tree request can choose its ordinary standalone root.
    pub(crate) fn without_context(&self) -> Self {
        Self {
            policy: self.policy.clone(),
            width: self.width.for_context(&self.policy),
            nerd_fonts: self.nerd_fonts,
            ..Self::default()
        }
    }
    pub(crate) fn hide_context(&mut self) {
        self.visible = false;
        self.edit();
    }
    pub(crate) fn chosen_root(&self) -> Option<&Path> {
        if self.explicit_root {
            self.root()
        } else {
            None
        }
    }

    /// Prepare a complete explicit browsing context before replacing the live
    /// one. Directory aliases use the same component rules as canonical files.
    pub(crate) fn opening_directory(&self, path: &Path, size: Size) -> io::Result<Self> {
        let root = crate::file::directory(path)?;
        let mut candidate = Self::configured(root, &self.policy, self.nerd_fonts, true);
        candidate.width = self.width.for_context(&self.policy);
        candidate.expand()?;
        candidate.focus(size)?;
        Ok(candidate)
    }

    /// Automatic file-parent rooting never replaces explicit directory intent.
    /// Keeping the same root also keeps its deliberate expansion/selection.
    pub(crate) fn opening_file(&self, path: &Path) -> io::Result<Option<Self>> {
        if self.explicit_root || self.policy.root == crate::config::TreeRoot::Cwd {
            return Ok(None);
        }
        let Some(parent) = path.parent().filter(|parent| Some(*parent) != self.root()) else {
            return Ok(None);
        };
        let mut candidate =
            Self::configured(parent.to_owned(), &self.policy, self.nerd_fonts, false);
        candidate.width = self.width.for_context(&self.policy);
        candidate.visible = self.visible;
        if candidate.visible {
            candidate.expand()?;
        }
        Ok(Some(candidate))
    }
}
