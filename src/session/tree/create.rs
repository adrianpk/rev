use crate::input::Focus;
use crate::tree::{CreateKind, Creation};

use super::super::{Action, Pending, Session};

impl Session {
    pub(super) fn tree_create(&mut self, kind: CreateKind) -> Action {
        if self.input.focus() != Focus::Editing {
            return self.message("a prompt owns input focus");
        }
        if !self.tree.focused() {
            return self.message("File tree is not focused");
        }
        let creation = match self.tree.prepare_create(kind) {
            Ok(creation) => creation,
            Err(error) => return self.message(&error.to_string()),
        };
        let label = match kind {
            CreateKind::File => "New file name: ",
            CreateKind::Directory => "New directory name: ",
        };
        if let Err(error) = self.input.argument(label, "") {
            return self.message(&error.to_string());
        }
        self.echo = format!("In {}", creation.directory().display());
        self.pending = Some(Pending::TreeCreate(creation));
        Action::Redraw
    }

    pub(in crate::session) fn tree_created(&mut self, creation: Creation, name: &str) -> Action {
        let path = match creation.publish(name) {
            Ok(path) => path,
            Err(error) => return self.message(&error.to_string()),
        };
        let notice = match self.tree.reveal_created(&path) {
            Ok(()) => format!("Created {}", path.display()),
            Err(error) => format!("Created {}; tree refresh failed: {error}", path.display()),
        };
        self.message(&notice)
    }

    pub(super) fn tree_open(&mut self) -> Action {
        if self.input.focus() != Focus::Editing {
            return self.message("a prompt owns input focus");
        }
        if !self.tree.focused() {
            return self.message("File tree is not focused");
        }
        let Some(entry) = self.tree.selected() else {
            return self.message("No tree entry selected");
        };
        self.input.suppress_repeat();
        if entry.is_directory() {
            return self.tree_navigation(crate::command::TreeCommand::Activate);
        }
        let path = entry.path().to_path_buf();
        let _ = self.load(&path);
        Action::Redraw
    }
}
