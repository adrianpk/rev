use std::io;
use std::path::{Path, PathBuf};

use crate::view::Size;

/// Workspace-side root and focus; editing views retain their own identities.
/// The root starts collapsed. Presentation performs no filesystem traversal.
#[derive(Default)]
pub struct Tree {
    root: Option<PathBuf>,
    visible: bool,
    focused: bool,
}

impl Tree {
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
        self.root = file.parent().map(Path::to_path_buf);
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
            self.root = Some(std::env::current_dir()?);
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

    pub(crate) fn line(&self, row: u16, height: u16) -> String {
        if row == 0 {
            return self.label();
        }
        if row + 1 == height {
            return if self.focused { "Tree *" } else { "Tree" }.into();
        }
        String::new()
    }

    fn label(&self) -> String {
        match self.root().and_then(Path::file_name) {
            Some(name) => format!("> {}/", name.to_string_lossy()),
            None => "> /".into(),
        }
    }
}
