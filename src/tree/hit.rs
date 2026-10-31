use std::path::PathBuf;

use super::Tree;

/// Opaque identity from one loaded preorder and viewport. Labels, indentation
/// and clipped text never supply an opening path or a replacement row index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    root: PathBuf,
    top: usize,
    index: usize,
    path: PathBuf,
    directory: bool,
    symlink: bool,
}

impl Tree {
    /// Resolve a tree-content-relative row through the same viewport as drawing.
    /// The caller checks content bounds; blank rows have no entry identity.
    pub fn hit(&self, row: u16) -> Option<Hit> {
        let entry = self.row_entry(row)?;
        Some(Hit {
            root: self.root.clone()?,
            top: self.top,
            index: self.top + usize::from(row),
            path: entry.path.clone(),
            directory: entry.directory,
            symlink: entry.symlink,
        })
    }

    /// Select only the retained root, viewport slot and entry identity together.
    /// A changed projection cannot substitute another row. This performs no IO.
    pub fn select_hit(&mut self, hit: &Hit) -> bool {
        if self.root.as_ref() != Some(&hit.root) || self.top != hit.top {
            return false;
        }
        let Some(entry) = self.entries.get(hit.index) else {
            return false;
        };
        if entry.path != hit.path
            || entry.directory != hit.directory
            || entry.symlink != hit.symlink
        {
            return false;
        }
        self.selected = hit.index;
        true
    }
}
