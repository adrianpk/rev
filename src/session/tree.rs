mod create;
mod delete;
mod rename;

use crate::command::{Command, TreeCommand};
use crate::input::Focus;
use crate::tree::Tree;
use crate::view::Size;

use super::{Action, Pending, Session};

impl Session {
    pub(super) fn follow_tree(&mut self) {
        let path = self
            .buffers
            .get(self.windows.selected().point.stamp.buffer)
            .ok()
            .filter(|_| !self.terminals.focused)
            .and_then(|buffer| buffer.file())
            .map(|file| file.path());
        self.tree.follow(self.windows.selected_id(), path);
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub(super) fn composition(&self) -> Option<crate::panel::Composition> {
        if self.startup_screen.is_some() {
            return self.source_composition(self.size);
        }
        let rows = self.terminal_rows();
        // Command candidates occupy the source prompt's former row. Keep the
        // terminal separator outside that region, including a single result.
        let command_separator = u16::from(rows.is_some() && self.input.palette().is_some());
        let size = Size {
            height: (self.size.height - rows.map_or(0, |r| r + 2))
                .saturating_sub(command_separator),
            ..self.size
        };
        let mut composition = self.source_composition(size)?;
        if let Some(rows) = rows {
            composition.attach_terminal(
                self.size,
                rows,
                self.terminals.focused && self.terminals.view.is_none(),
            )?;
        }
        Some(composition)
    }

    fn source_composition(&self, size: Size) -> Option<crate::panel::Composition> {
        let hidden = crate::tree::Tree::default();
        let tree = if self.startup_screen.is_some() {
            &hidden
        } else {
            &self.tree
        };
        if let Some(names) = self.input.names() {
            return crate::panel::Composition::completion(
                &self.windows,
                tree,
                size,
                names.requested_rows(),
            );
        }
        if let Some(palette) = self.input.palette() {
            return crate::panel::Composition::completion(
                &self.windows,
                tree,
                size,
                palette.requested_rows(),
            );
        }
        if let Some(path) = self.path_navigation() {
            return crate::panel::Composition::completion(
                &self.windows,
                tree,
                size,
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
        crate::panel::Composition::workspace(&self.windows, tree, size, rows)
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
        if let Some(command) = TreeCommand::alias(command) {
            return Some(self.tree_command(command));
        }
        if matches!(
            command,
            Command::Tree(_)
                | Command::Quit
                | Command::MouseMode
                | Command::FindFile
                | Command::GenerateNewBuffer
                | Command::OpenDirectory
                | Command::ContextClose
                | Command::DashboardOpen
                | Command::ProjectCreate
                | Command::ProjectPromote
                | Command::ProjectSwitch
                | Command::ProjectList
                | Command::WorkspaceCreate
                | Command::WorkspaceOpen
                | Command::WorkspaceList
                | Command::WorkspaceAddDirectory
                | Command::WorkspaceAddProject
                | Command::WorkspaceRemoveRoot
                | Command::ProjectRename
                | Command::ProjectForget
                | Command::ProjectSetRoot
                | Command::ProjectPromoteToWorkspace
                | Command::ProjectDuplicateAsWorkspace
                | Command::WorkspaceRename
                | Command::WorkspaceForget
                | Command::WorkspaceSetRootPath
        ) {
            return None;
        }
        match command {
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
        let result = self.navigate_tree(command);
        match result {
            Ok(()) => match self.tree.diagnostic() {
                Some(message) => self.notice(message),
                None => self.message("File tree updated"),
            },
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn navigate_tree(&mut self, command: TreeCommand) -> std::io::Result<()> {
        match command {
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
            TreeCommand::UpDirectory => {
                self.input.suppress_repeat();
                self.tree.up_directory()
            }
            _ => unreachable!("presentation commands route before tree navigation"),
        }
    }
}
