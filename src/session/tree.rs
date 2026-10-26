use crate::command::{Command, TreeCommand};
use crate::input::Focus;
use crate::tree::Tree;
use crate::view::Size;

use super::{Action, Pending, Session};

impl Session {
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub(super) fn editing_size(&self) -> Size {
        let width = self.tree.width(self.size);
        Size {
            width: self.size.width - width - u16::from(width > 0),
            ..self.size
        }
    }

    pub(super) fn tree_command(&mut self, command: TreeCommand) -> Action {
        if self.input.focus() != Focus::Editing {
            return self.message("a prompt owns input focus");
        }
        let result = match command {
            TreeCommand::Toggle => self.tree.toggle(self.size),
            TreeCommand::Focus => self.tree.focus(self.size),
        };
        if let Err(error) = result {
            return self.message(&format!("Open file tree: {error}"));
        }
        self.tree_notice()
    }

    fn tree_notice(&mut self) -> Action {
        let message = if self.tree.focused() {
            "File tree focused"
        } else if self.tree.visible() {
            "File tree hidden at this width"
        } else {
            "File tree hidden"
        };
        self.message(message)
    }

    /// Workspace focus wraps transient prompt focus. M-x can run in the tree;
    /// canceling that prompt returns to the retained workspace owner.
    pub(super) fn tree_input(&mut self, command: Command) -> Option<Action> {
        if !self.tree.focused() {
            return None;
        }
        match command {
            Command::Tree(_) | Command::Quit => None,
            Command::KeyboardQuit if matches!(self.pending, Some(Pending::Command)) => {
                self.pending = None;
                self.input.cancel();
                Some(self.message("Quit"))
            }
            Command::KeyboardQuit | Command::InsertTab => {
                self.tree.edit();
                self.input.cancel();
                Some(self.message("Editing"))
            }
            _ => Some(self.message("File tree owns input focus")),
        }
    }
}
