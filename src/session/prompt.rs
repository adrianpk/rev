use crate::command::Command;
use crate::input::Focus;

use super::{Action, Pending, Session};

impl Session {
    pub(super) fn prepare_command(&mut self, command: Command) -> Option<Action> {
        if command == Command::ExecuteExtended {
            return Some(self.extended());
        }
        if command == Command::KeyboardQuit {
            self.restore_search();
        }
        self.tree_input(command)
    }

    pub(super) fn command_outcome(&mut self, command: Command) -> Action {
        // C-g consumes pending focus in navigation without resetting the view.
        // Accepted names retire that stage before the shared handler takes over.
        if command != Command::KeyboardQuit {
            self.retire_command();
        }
        self.execute(command)
    }

    pub(super) fn retire_command(&mut self) {
        if self.input.focus() == Focus::Editing && matches!(self.pending, Some(Pending::Command)) {
            self.pending = None;
        }
    }

    pub(super) fn complete_argument(&mut self) -> Action {
        match self.pending {
            Some(Pending::Buffer { .. }) => self.complete_buffer(),
            Some(Pending::Argument(_) | Pending::Write { .. }) => {
                self.message("Enter a path; C-g cancels")
            }
            _ => self.message("No argument is active"),
        }
    }
}
