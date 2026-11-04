use super::{Action, Session, SessionId};
use crate::session::Pending;

impl Session {
    pub(super) fn list_terminals(&mut self) -> Action {
        let selected = self
            .terminal_buffer()
            .or_else(|| self.buffers.iter().find(|b| b.terminal().is_some()))
            .map(|b| b.id());
        let Some(selected) = selected else {
            return self.message("No terminal sessions");
        };
        if let Err(error) = self.input.inventory() {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::Terminals(selected));
        self.message("Terminals: arrows move, Enter selects, C-g cancels")
    }

    pub(super) fn rename_terminal(&mut self) -> Action {
        let Some(id) = self.current_terminal() else {
            return self.error("No terminal is selected");
        };
        if let Err(error) = self.input.argument("Rename terminal: ", "") {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::TerminalRename(id));
        self.message("Enter a unique terminal buffer name")
    }

    pub(in crate::session) fn terminal_renamed(&mut self, id: SessionId, name: &str) -> Action {
        self.input.cancel();
        let Some(buffer) = self
            .buffers
            .iter()
            .find(|b| b.terminal() == Some(id))
            .map(|b| b.id())
        else {
            return self.error("Terminal buffer is no longer live");
        };
        match self.buffers.rename_terminal(buffer, name) {
            Ok(()) => self.message("Terminal renamed"),
            Err(error) => self.error(&error),
        }
    }
}
