mod entry;
mod navigation;
mod refresh;

pub use entry::Entry;

use std::io;
use std::path::{Path, PathBuf};

use crate::view::Size;

/// Workspace-side root and focus; editing views retain their own identities.
/// The root starts collapsed. Filesystem reads happen only on explicit expansion
/// or refresh; drawing and visible movement never read directories.
#[derive(Default)]
pub struct Tree {
    root: Option<PathBuf>,
    visible: bool,
    focused: bool,
    entries: Vec<Entry>,
    selected: usize,
    top: usize,
}

impl Tree {
    /// Create a retained collapsed root without reading its directory.
    pub fn new(root: PathBuf) -> Self {
        Self {
            entries: vec![Entry::root(root.clone())],
            root: Some(root),
            ..Self::default()
        }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn selected(&self) -> Option<&Entry> {
        self.entries.get(self.selected)
    }

    pub(crate) fn reveal(&mut self, size: Size) {
        let height = usize::from(size.height.saturating_sub(2));
        if height == 0 {
            return;
        }
        self.top = self.top.min(self.selected);
        if self.selected - self.top >= height {
            self.top = self.selected + 1 - height;
        }
    }

    pub(crate) fn selected_row(&self) -> u16 {
        (self.selected - self.top) as u16
    }

    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn width(&self, size: Size) -> u16 {
        if !self.visible || size.width < 32 || !size.usable() {
            return 0;
        }
        24.min(size.width / 3)
    }

    pub(crate) fn initial(&mut self, file: &Path) {
        if let Some(parent) = file.parent() {
            *self = Self::new(parent.to_path_buf());
        }
    }

    pub(crate) fn toggle(&mut self, size: Size) -> io::Result<()> {
        if self.visible {
            self.visible = false;
            self.edit();
        } else {
            self.focus(size)?;
        }
        Ok(())
    }

    pub(crate) fn focus(&mut self, size: Size) -> io::Result<()> {
        if self.root.is_none() {
            *self = Self::new(std::env::current_dir()?);
        }
        self.visible = true;
        self.focused = self.width(size) > 0;
        Ok(())
    }

    pub(crate) fn edit(&mut self) {
        self.focused = false;
    }

    pub(crate) fn resize(&mut self, size: Size) -> bool {
        let hidden = self.focused && self.width(size) == 0;
        if hidden {
            self.edit();
        }
        hidden
    }

    pub(crate) fn line(&self, row: u16, height: u16, width: u16) -> String {
        if row + 1 == height {
            return if self.focused { "Tree *" } else { "Tree" }.into();
        }
        self.entries
            .get(self.top + usize::from(row))
            .map(|entry| entry.label(width))
            .unwrap_or_default()
    }
}
