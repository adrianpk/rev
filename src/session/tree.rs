mod create;
mod delete;
mod rename;

use crate::command::{Command, Motion, Scroll, TreeCommand};
use crate::input::Focus;
use crate::tree::Tree;
use crate::view::Size;

use super::{Action, Pending, Session};

impl Session {
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub(super) fn composition(&self) -> Option<crate::panel::Composition> {
        if let Some(palette) = self.input.palette() {
            return crate::panel::Composition::completion(
                &self.windows,
                &self.tree,
                self.size,
                palette.requested_rows(),
            );
        }
        if let Some(path) = self.path_navigation() {
            return crate::panel::Composition::completion(
                &self.windows,
                &self.tree,
                self.size,
                path.requested_rows(
                    self.buffers
                        .snapshot()
                        .settings()
                        .minibuffer
                        .candidate_limit,
                ),
            );
        }
        let rows = if self.timestamp_calendar().is_some() {
            8
        } else {
            0
        };
        crate::panel::Composition::workspace(&self.windows, &self.tree, self.size, rows)
    }

    pub(super) fn editing_size(&self) -> Size {
        self.composition()
            .map_or(self.size, |composition| composition.editing_size())
    }

    pub(super) fn tree_command(&mut self, command: TreeCommand) -> Action {
        if self.input.focus() != Focus::Editing {
            return self.message("a prompt owns input focus");
        }
        self.tree_entry(command)
    }

    fn tree_entry(&mut self, command: TreeCommand) -> Action {
        match command {
            TreeCommand::Delete => return self.tree_delete(),
            TreeCommand::Rename => return self.tree_rename(),
            TreeCommand::Open => return self.tree_open(),
            TreeCommand::CreateFile => return self.tree_create(crate::tree::CreateKind::File),
            TreeCommand::CreateDirectory => {
                return self.tree_create(crate::tree::CreateKind::Directory);
            }
            _ => {}
        }
        if !matches!(command, TreeCommand::Toggle | TreeCommand::Focus) {
            return self.tree_navigation(command);
        }
        let result = match command {
            TreeCommand::Toggle => self.tree.toggle(self.size),
            _ => self.tree.focus(self.size),
        };
        if let Err(error) = result {
            return self.error(&format!("Open file tree: {error}"));
        }
        self.tree_notice()
    }

    fn tree_notice(&mut self) -> Action {
        if let Some(message) = self.tree.diagnostic() {
            return self.notice(message);
        }
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
        if let Some(command) = tree_alias(command) {
            return Some(self.tree_command(command));
        }
        match command {
            Command::Tree(_) | Command::Quit | Command::MouseMode | Command::FindFile => None,
            Command::KeyboardQuit
                if matches!(
                    self.pending,
                    Some(
                        Pending::Command
                            | Pending::Path(_)
                            | Pending::TreeCreate(_)
                            | Pending::TreeRename(_)
                            | Pending::TreeDelete(_)
                    )
                ) =>
            {
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

impl Session {
    fn tree_navigation(&mut self, command: TreeCommand) -> Action {
        if !self.tree.focused() {
            return self.message("File tree is not focused");
        }
        let result = match command {
            TreeCommand::Previous => {
                self.tree.move_selection(false);
                Ok(())
            }
            TreeCommand::Next => {
                self.tree.move_selection(true);
                Ok(())
            }
            TreeCommand::Expand => self.tree.expand(),
            TreeCommand::Collapse => {
                self.tree.collapse();
                Ok(())
            }
            TreeCommand::Activate => {
                self.input.suppress_repeat();
                self.tree.activate()
            }
            TreeCommand::Refresh => {
                self.input.suppress_repeat();
                self.tree.refresh()
            }
            _ => unreachable!("presentation commands route before tree navigation"),
        };
        match result {
            Ok(()) => match self.tree.diagnostic() {
                Some(message) => self.notice(message),
                None => self.message("File tree updated"),
            },
            Err(error) => self.error(&error.to_string()),
        }
    }
}

fn tree_alias(command: Command) -> Option<TreeCommand> {
    match command {
        Command::Scroll(Scroll::PreviousLine) => Some(TreeCommand::Previous),
        Command::Scroll(Scroll::NextLine) => Some(TreeCommand::Next),
        Command::Motion(Motion::ForwardChar) => Some(TreeCommand::Expand),
        Command::Motion(Motion::BackwardChar) => Some(TreeCommand::Collapse),
        Command::Newline => Some(TreeCommand::Open),
        _ => None,
    }
}
