//! Explicit file lifecycle shares native save/conflict/close ownership.
use super::Session;
use crate::{
    buffer::Save,
    file,
    plugins::lifecycle::editor::{Reply, Request},
};
use rev_plugin_sdk::{
    methods::*,
    rpc::{ErrorKind, RpcError},
};
fn error(kind: ErrorKind) -> RpcError {
    RpcError::application(kind)
}
impl Session {
    pub(super) fn plugin_save(
        &mut self,
        request: &Request,
        params: BuffersSaveParams,
        target: Option<file::Location>,
    ) -> Result<Reply, RpcError> {
        let id = self.plugin_buffer(&params.buffer)?;
        let buffer = self.buffers.get(id).expect("resolved source");
        let descriptor = self.plugin_descriptor(buffer)?;
        if descriptor.stamp.revision != params.expected_revision {
            return Err(error(ErrorKind::StaleRevision));
        }
        if buffer.is_readonly() {
            return Err(error(ErrorKind::ReadOnly));
        }
        if self.macros.active() || self.active_clock.as_ref().is_some_and(|c| c.buffer() == id) {
            return Err(error(ErrorKind::UnsupportedInvocation));
        }
        if buffer.len_bytes() > 16 * 1024 * 1024 {
            return Err(error(ErrorKind::LimitExceeded));
        }
        let location = match target {
            Some(target)
                if buffer
                    .file()
                    .is_none_or(|prior| prior.path() != target.path()) =>
            {
                if params.expected_disk != DiskObservation::Absent || target.is_present() {
                    return Err(error(ErrorKind::ConfirmationRequired));
                }
                // Retargeting existing bookmark records requires their native
                // loading/consent workflow. A plugin cannot invent that consent.
                if buffer.file().is_some() {
                    return Err(error(ErrorKind::ConfirmationRequired));
                }
                target
            }
            _ => {
                if params.expected_disk != descriptor.expected_disk {
                    return Err(error(ErrorKind::StateConflict));
                }
                buffer
                    .file()
                    .cloned()
                    .ok_or_else(|| error(ErrorKind::ConfirmationRequired))?
            }
        };
        rev_plugin_sdk::methods::Label::new(
            location
                .path()
                .to_str()
                .ok_or_else(|| error(ErrorKind::UnsupportedResource))?,
        )?;
        let ingress = request.retain(buffer.len_bytes())?;
        let save = self
            .buffers
            .prepare_resolved_save(id, location)
            .map_err(|_| error(ErrorKind::StateConflict))?;
        if !request.live() {
            return Err(error(ErrorKind::Cancelled));
        }
        Ok(Reply::Stage(save, ingress))
    }
    pub(super) fn plugin_publish_save(
        &mut self,
        request: &Request,
        save: Save,
        staged: file::Staged,
    ) -> Result<Reply, RpcError> {
        let id = save.stamp().buffer;
        let buffer = self
            .buffers
            .get(id)
            .map_err(|_| error(ErrorKind::MissingBuffer))?;
        if buffer.stamp() != save.stamp() {
            return Err(error(ErrorKind::StaleRevision));
        }
        if buffer.is_readonly() {
            return Err(error(ErrorKind::ReadOnly));
        }
        if self.macros.active() || self.active_clock.as_ref().is_some_and(|c| c.buffer() == id) {
            return Err(error(ErrorKind::UnsupportedInvocation));
        }
        if !request.live() {
            return Err(error(ErrorKind::Cancelled));
        }
        let state = save.state();
        let outcome = self.buffers.publish_staged_save(save, staged);
        self.note_refile_save(state, &outcome);
        self.lsp_saved(&outcome);
        if !outcome.is_complete() {
            return Err(error(ErrorKind::StateConflict));
        }
        let descriptor =
            self.plugin_descriptor(self.buffers.get(id).expect("saved live source"))?;
        Ok(Reply::Saved(BuffersSaveResult {
            stamp: descriptor.stamp,
            expected_disk: descriptor.expected_disk,
        }))
    }
    pub(super) fn plugin_close(
        &mut self,
        request: &Request,
        params: BuffersCloseParams,
    ) -> Result<Reply, RpcError> {
        let id = self.plugin_buffer(&params.buffer)?;
        let buffer = self.buffers.get(id).expect("resolved source");
        if self.plugin_descriptor(buffer)?.stamp.revision != params.expected_revision {
            return Err(error(ErrorKind::StaleRevision));
        }
        if buffer.is_modified() || self.active_clock.as_ref().is_some_and(|c| c.buffer() == id) {
            return Err(error(ErrorKind::ConfirmationRequired));
        }
        if self.pending.is_some()
            || self.input.focus() != crate::input::Focus::Editing
            || self.macros.active()
            || self.capture_blocks_input()
            || self.capture_selected()
            || self.source_edit_selected()
            || self.windows.retained_view_active()
            || self.tangling.is_some()
            || self.execution.is_some()
            || self.feeds.is_some()
        {
            return Err(error(ErrorKind::UnsupportedInvocation));
        }
        let kill = self
            .buffers
            .prepare_kill(id)
            .map_err(|_| error(ErrorKind::MissingBuffer))?;
        if !request.live() {
            return Err(error(ErrorKind::Cancelled));
        }
        // The native removal path updates every shared view and completion/LSP
        // owner. It preserves prompts; close_buffer would cancel native input.
        self.remove_buffer(kill);
        if self.buffers.get(id).is_ok() {
            return Err(error(ErrorKind::StateConflict));
        }
        Ok(Reply::Closed(BuffersCloseResult { closed: true }))
    }
}
