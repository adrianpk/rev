mod create;
mod delete;
mod entry;
mod follow;
mod hit;
mod navigation;
mod presentation;
mod refresh;
mod rename;
mod root;
mod viewport;
mod width;
mod workspace;

pub use create::{CreateKind, Creation};
pub use entry::Entry;
pub use hit::Hit;
pub use presentation::Kind;

use std::io;
use std::path::{Path, PathBuf};

use crate::view::Size;

/// Workspace-side root and focus; editing views retain their own identities.
/// First presentation initializes immediate children once. Filesystem reads
/// happen on presentation, expansion or refresh; drawing/movement perform none.
#[derive(Default)]
pub struct Tree {
    root: Option<PathBuf>,
    workspace: Option<(crate::context::WorkspaceId, Vec<crate::context::Root>)>,
    visible: bool,
    focused: bool,
    entries: Vec<Entry>,
    selected: usize,
    top: usize,
    policy: crate::config::Tree,
    width: width::Width,
    viewport: viewport::Viewport,
    nerd_fonts: bool,
    explicit_root: bool,
    initialized: bool,
    focus_pending: bool,
    active_view: Option<crate::window::ViewId>,
    active_path: Option<PathBuf>,
    active: Option<usize>,
    follow_dirty: bool,
    follow_pending: bool,
}

impl Tree {
    /// Create a retained collapsed root without reading its directory.
    pub fn new(root: PathBuf) -> Self {
        Self::configured(root, &crate::config::Tree::default(), false, false)
    }

    /// Observe no entries until explicit expansion or refresh; the supplied
    /// root and admitted policy determine initial visibility without focus.
    pub fn with_settings(root: PathBuf, settings: &crate::config::Snapshot) -> Self {
        Self::configured(
            root,
            &settings.settings().tree,
            settings.settings().gutter.nerd_fonts,
            false,
        )
    }

    fn configured(
        root: PathBuf,
        policy: &crate::config::Tree,
        nerd_fonts: bool,
        explicit_root: bool,
    ) -> Self {
        let mut tree = Self {
            policy: policy.clone(),
            width: width::Width::configured(policy),
            nerd_fonts,
            explicit_root,
            visible: policy.visible,
            ..Self::default()
        };
        tree.set_root(root);
        tree
    }

    pub(crate) fn startup(
        policy: &crate::config::Tree,
        file: Option<&Path>,
        nerd_fonts: bool,
    ) -> io::Result<Self> {
        let parent = file
            .and_then(Path::parent)
            .filter(|_| policy.root != crate::config::TreeRoot::Cwd);
        let root = match parent {
            Some(parent) => parent.to_path_buf(),
            None => std::env::current_dir()?,
        };
        Ok(Self::configured(root, policy, nerd_fonts, false))
    }

    fn set_root(&mut self, root: PathBuf) {
        self.entries = self.root_entries(root.clone());
        self.root = Some(root);
        self.selected = 0;
        self.top = 0;
        self.initialized = false;
        self.follow_dirty = true;
        self.new_width_context();
    }

    fn children_start(&self, index: usize) -> usize {
        index
            + 1
            + usize::from(
                self.entries
                    .get(index + 1)
                    .is_some_and(|entry| entry.parent),
            )
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
        let selected = if self.focused || !self.policy.follow_active_buffer {
            self.selected
        } else {
            if !self.follow_pending {
                return;
            }
            self.follow_pending = false;
            let Some(active) = self.active else {
                return;
            };
            active
        };
        self.top = self.top.min(selected);
        if selected - self.top >= height {
            self.top = selected + 1 - height;
        }
    }

    pub(crate) fn selected_row(&self) -> Option<u16> {
        self.selected
            .checked_sub(self.top)
            .and_then(|row| u16::try_from(row).ok())
    }

    pub fn root(&self) -> Option<&Path> {
        if let Some((_, roots)) = &self.workspace {
            let id = self.selected()?.root_id.as_ref()?;
            return roots.iter().find(|r| &r.id == id).map(|r| r.path.as_path());
        }
        self.root.as_deref()
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
    pub fn focused(&self) -> bool {
        self.focused
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Tree geometry intentionally floors the admitted terminal fraction using saturating float-to-cell conversion."
    )]
    pub fn width(&self, size: Size) -> u16 {
        if !self.visible || size.width < 32 || !size.usable() {
            return 0;
        }
        let maximum = (f64::from(size.width) * self.policy.max_fraction).floor() as u16;
        self.width.desired.min(maximum)
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
        if self.root.is_none() && self.workspace.is_none() {
            self.set_root(std::env::current_dir()?);
        }
        self.initialize()?;
        self.visible = true;
        self.focused = self.width(size) > 0;
        self.focus_pending = !self.focused;
        Ok(())
    }

    fn initialize(&mut self) -> io::Result<()> {
        if !self.initialized {
            self.expand()?;
            self.initialized = true;
        }
        Ok(())
    }

    pub(crate) fn present(&mut self) -> io::Result<()> {
        if self.visible {
            self.initialize()?;
        }
        Ok(())
    }

    /// Report incomplete retained directory observations without further IO.
    pub fn diagnostic(&self) -> Option<&'static str> {
        if self.entries.iter().any(|e| e.unavailable) {
            return Some("Workspace contains unavailable roots; membership is retained");
        }
        self.entries
            .iter()
            .any(|entry| entry.incomplete)
            .then_some("Incomplete tree listing: directory observation limit reached")
    }

    pub(crate) fn edit(&mut self) {
        self.follow_dirty |= self.focused || self.focus_pending;
        self.focused = false;
        self.focus_pending = false;
    }

    pub(crate) fn resize(&mut self, size: Size) -> bool {
        let hidden = self.focused && self.width(size) == 0;
        if hidden {
            self.focused = false;
            self.focus_pending = true;
        } else if self.focus_pending && self.width(size) > 0 {
            self.focus_pending = false;
            self.focused = true;
        }
        self.follow_dirty |= hidden || self.follow_pending;
        hidden
    }

    pub(crate) fn status(&self) -> &str {
        if self.workspace.is_some() {
            if self.focused {
                "Workspace *"
            } else {
                "Workspace"
            }
        } else if self.focused {
            "Tree *"
        } else {
            "Tree"
        }
    }

    fn row_entry(&self, row: u16) -> Option<&Entry> {
        self.entries.get(self.top + usize::from(row))
    }

    pub(crate) fn row_kind(&self, row: u16) -> Option<Kind> {
        self.row_entry(row)
            .filter(|_| self.policy.decorated)
            .map(Entry::kind)
    }

    pub(crate) fn line(&self, row: u16) -> Result<String, crate::coordinates::Error> {
        match self.row_entry(row) {
            Some(entry) => entry.label(self.policy.decorated, self.nerd_fonts),
            None if self.workspace.is_some() && self.entries.is_empty() && row == 0 => {
                Ok("No roots; workspace-add-directory".into())
            }
            None => Ok(String::new()),
        }
    }
}
