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
        let id = self.windows.selected().point.stamp.buffer;
        match command {
            LspCommand::Mode => self.lsp.set_enabled(!self.lsp.enabled()),
            LspCommand::Disconnect => {
                if let Ok(buffer) = self.buffers.get_mut(id) {
                    self.lsp.exclude(buffer);
                }
            }
            LspCommand::Connect => self.lsp.restore(id),
            LspCommand::Restart => {
                if let Ok(buffer) = self.buffers.get_mut(id) {
                    self.lsp.restart_buffer(buffer, self.tree.chosen_root());
                }
            }
            LspCommand::Status => {}
        }
        self.activate_lsp();
        self.message(&format!("LSP: {}", status_text(&self.lsp_status())))
    }

    /// Status observes transport readiness, independently of native completion.
    pub fn lsp_status(&self) -> Status {
        self.lsp
            .buffer_status(self.windows.selected().point.stamp.buffer)
    }

    /// The synchronized identity is valid only for its exact canonical revision.
    pub fn lsp_document_version(&self) -> Option<crate::lsp::DocumentVersion> {
        let buffer = self.buffer().ok()?;
        self.lsp
            .document_version(buffer.id())
            .filter(|version| version.stamp == buffer.stamp())
    }

    pub(super) fn activate_lsp(&mut self) {
        if !self.lsp.enabled() {
            return;
        }
        let id = self.windows.selected().point.stamp.buffer;
        if let Ok(buffer) = self.buffers.get_mut(id) {
            self.lsp.attach_buffer(buffer, self.tree.chosen_root());
        }
    }

    pub(super) fn lsp_saved(&mut self, outcome: &crate::file::Outcome) {
        if !outcome.is_complete() || !self.lsp.enabled() {
            return;
        }
        let id = self
            .buffers
            .iter()
            .find(|buffer| {
                buffer
                    .file()
                    .is_some_and(|file| file.path() == outcome.target())
            })
            .map(|buffer| buffer.id());
        if let Some(id) = id
            && let Ok(buffer) = self.buffers.get_mut(id)
        {
            self.lsp.saved_buffer(buffer, self.tree.chosen_root());
        }
    }

    pub(super) fn lsp_remaining(&self) -> Option<Duration> {
        self.lsp.starting().then_some(Duration::from_millis(10))
    }

    pub(super) fn expire_lsp(&mut self) -> Action {
        self.lsp.collect();
        self.activate_lsp();
        Action::Wait
    }
}

fn status_text(status: &Status) -> String {
    match status {
        Status::Disabled => "disabled".into(),
        Status::Excluded => "excluded for this buffer".into(),
        Status::Unavailable => "no configured server for this file".into(),
        Status::Starting => "starting".into(),
        Status::Ready(_) => "ready (transport)".into(),
        Status::Missing(command) => format!("missing executable: {command}"),
        Status::Failed(error) => format!("failed: {error}"),
        Status::Limited => "server limit reached".into(),
        Status::Stopping => "stopping".into(),
    }
}
