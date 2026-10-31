use super::{Action, Session};
use crate::command::{Command, LspCommand};
use crate::lsp::Status;
use std::time::Duration;

impl Session {
    pub(super) fn lsp_command(&mut self, command: Command) -> Option<Action> {
        let Command::Lsp(command) = command else {
            return None;
        };
        Some(self.lsp_control(command))
    }

    fn lsp_control(&mut self, command: LspCommand) -> Action {
        if command == LspCommand::Mode {
            self.lsp.set_enabled(!self.lsp.enabled());
            self.lsp_buffer = None;
        }
        self.activate_lsp();
        self.message(&format!("LSP: {}", status_text(&self.lsp.status())))
    }

    /// Status observes transport readiness, independently of native completion.
    pub fn lsp_status(&self) -> Status {
        self.lsp.status()
    }

    pub(super) fn activate_lsp(&mut self) {
        if !self.lsp.enabled() {
            return;
        }
        let buffer = self.windows.selected().point.stamp.buffer;
        if self.lsp_buffer == Some(buffer) {
            return;
        }
        self.lsp_buffer = Some(buffer);
        let Ok(buffer) = self.buffers.get(buffer) else {
            return;
        };
        self.lsp.clear_selection();
        if let Some(file) = buffer.file() {
            self.lsp.activate(
                file.path(),
                buffer.syntax_language(),
                self.tree.chosen_root(),
            );
        }
    }

    pub(super) fn lsp_remaining(&self) -> Option<Duration> {
        self.lsp.starting().then_some(Duration::from_millis(10))
    }

    pub(super) fn expire_lsp(&mut self) -> Action {
        if self.lsp.collect() {
            self.lsp_buffer = None;
        }
        self.activate_lsp();
        Action::Wait
    }
}

fn status_text(status: &Status) -> String {
    match status {
        Status::Disabled => "disabled".into(),
        Status::Unavailable => "no configured server for this file".into(),
        Status::Starting => "starting".into(),
        Status::Ready(_) => "ready (transport)".into(),
        Status::Missing(command) => format!("missing executable: {command}"),
        Status::Failed(error) => format!("failed: {error}"),
        Status::Limited => "server limit reached".into(),
        Status::Stopping => "stopping".into(),
    }
}
