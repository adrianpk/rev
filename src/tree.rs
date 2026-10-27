mod create;
mod delete;
mod entry;
mod navigation;
mod refresh;
mod rename;

pub use create::{CreateKind, Creation};
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
    policy: crate::config::Tree,
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

    /// Observe no entries until explicit expansion or refresh; the supplied
    /// root and admitted policy determine initial visibility without focus.
    pub fn with_settings(root: PathBuf, settings: &crate::config::Snapshot) -> Self {
        Self::configured(root, &settings.settings().tree)
    }

    fn configured(root: PathBuf, policy: &crate::config::Tree) -> Self {
        Self {
            policy: policy.clone(),
            visible: policy.visible,
            ..Self::new(root)
        }
    }

    pub(crate) fn startup(policy: &crate::config::Tree, file: Option<&Path>) -> io::Result<Self> {
        let parent = file
            .and_then(Path::parent)
            .filter(|_| policy.root != crate::config::TreeRoot::Cwd);
        let root = match parent {
            Some(parent) => parent.to_path_buf(),
            None => std::env::current_dir()?,
        };
        Ok(Self::configured(root, policy))
    }

    fn set_root(&mut self, root: PathBuf) {
        self.entries = vec![Entry::root(root.clone())];
        self.root = Some(root);
        self.selected = 0;
        self.top = 0;
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn selected(&self) -> Option<&Entry> {
        self.entries.get(self.selected)
    }

    pub(crate) fn reveal(&mut self, rows: u16) {
        let height = usize::from(rows);
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
        let maximum = (f64::from(size.width) * self.policy.max_fraction).floor() as u16;
        (self.policy.width as u16).min(maximum)
    }

    pub(crate) fn initial(&mut self, file: &Path) {
        if self.policy.root != crate::config::TreeRoot::Cwd {
            if let Some(parent) = file.parent() {
                self.set_root(parent.to_path_buf());
            }
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
            self.set_root(std::env::current_dir()?);
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

    pub(crate) fn status(&self) -> &str {
        if self.focused { "Tree *" } else { "Tree" }
    }

    pub(crate) fn line(&self, row: u16, width: u16) -> String {
        self.entries
            .get(self.top + usize::from(row))
            .map(|entry| entry.label(width))
            .unwrap_or_default()
    }
}
