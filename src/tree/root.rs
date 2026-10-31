use std::io;
use std::path::Path;

use super::Tree;
use crate::view::Size;

impl Tree {
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
        let mut candidate = Self::configured(root, &self.policy);
        candidate.explicit_root = true;
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
        let mut candidate = Self::configured(parent.to_owned(), &self.policy);
        candidate.visible = self.visible;
        if candidate.visible {
            candidate.expand()?;
        }
        Ok(Some(candidate))
    }
}
