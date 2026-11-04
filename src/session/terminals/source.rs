use super::super::{Action, Session};
use crate::{command::Command, input::Focus};

impl Session {
    /// A lower-panel terminal is a resource focus, like the native tree. It
    /// cannot borrow the selected source pane's file, point, mode or providers.
    pub(in crate::session) fn terminal_source_guard(&mut self, command: Command) -> Option<Action> {
        if !self.terminals.focused && self.buffer().is_ok_and(|b| b.terminal().is_none()) {
            return None;
        }
        if matches!(
            command,
            Command::Terminal(_)
                | Command::ExecuteExtended
                | Command::KeyboardQuit
                | Command::Quit
                | Command::Buffer(_)
                | Command::Split(_)
                | Command::Window(_)
        ) {
            return None;
        }
        Some(self.error("Command requires a source buffer; C-g returns to the editor"))
    }

    pub(in crate::session) fn terminal_quit_input(&mut self) -> Option<Action> {
        if let Some(action) = self.terminal_copy_quit() {
            return Some(action);
        }
        if !self.terminals.focused {
            return None;
        }
        if !self.terminals.host_cancel
            && self.input.focus() == Focus::Editing
            && self.pending.is_none()
        {
            self.terminal_return();
        }
        self.input.cancel();
        Some(self.message("Quit"))
    }
}
