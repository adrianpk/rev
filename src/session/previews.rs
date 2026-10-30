use super::{Action, Session, outline::Request};
use crate::{
    command::{Command, OrgCommand as C},
    input::Focus,
    org::{
        links::Snapshot,
        preview::{Output, Work},
    },
};
impl Session {
    pub(super) fn preview_command(&mut self, command: Command) -> Option<Action> {
        if command == Command::KeyboardQuit && self.preview_request.is_some() {
            self.preview_request = None;
            self.org.cancel_preview();
            return Some(self.message("Preview cancelled; source unchanged"));
        }
        let Command::Org(
            command @ (C::LinkPreview
            | C::LinkPreviewClear
            | C::LinkPreviewRefresh
            | C::LinkPreviewRegion),
        ) = command
        else {
            return None;
        };
        Some(match self.begin_preview(command) {
            Ok(()) => Action::Redraw,
            Err(e) => self.error(&e),
        })
    }
    fn begin_preview(&mut self, command: C) -> Result<(), String> {
        if self.input.focus() != Focus::Editing
            || self.tree.focused()
            || self.windows.retained_view_active()
        {
            return Err("Editing owns preview commands".into());
        }
        if command == C::LinkPreviewClear {
            self.windows.selected_mut().preview.clear();
            self.windows.selected_mut().preview.enabled = false;
            self.preview_request = None;
            self.org.cancel_preview();
            self.message("Org previews cleared; source unchanged");
            return Ok(());
        }
        self.admit_preview(command)?;
        let policy = &self.buffers.snapshot().settings().org;
        let source = Snapshot::new(
            self.buffer().map_err(|e| e.to_string())?,
            self.view().point,
            policy,
        )?;
        let work = Work::new(
            source,
            command != C::LinkPreview,
            false,
            policy.preview.max_bytes,
        );
        let seconds = policy.limits.job_seconds;
        self.windows.selected_mut().preview.enabled = true;
        self.preview_request = Some(Request {
            view: self.windows.selected_id(),
            origin: self.view().clone(),
            command,
        });
        self.org.start_preview(work, seconds)?;
        self.message("Checking local previews; C-g cancels; no network fetch");
        Ok(())
    }
    fn admit_preview(&self, command: C) -> Result<(), String> {
        if self.org.busy() || self.pending.is_some() || self.source_edit.is_some() {
            return Err("Another Org interaction owns input; C-g cancels".into());
        }
        if !self.buffer().is_ok_and(|b| b.is_org()) {
            return Err("Native Org mode is not active".into());
        }
        if command == C::LinkPreviewRefresh && !self.view().preview.enabled {
            return Err("Previews are disabled; org-link-preview enables this view".into());
        }
        Ok(())
    }
    pub(super) fn finish_preview(&mut self, result: Result<Output, String>) {
        let Some(request) = self.preview_request.take() else {
            return;
        };
        if !self.org_source_current(&request) {
            self.error("Preview source or view changed; old result discarded");
            return;
        }
        match result.and_then(|out| {
            out.current()?;
            Ok(out)
        }) {
            Ok(output) => self.publish_preview(output),
            Err(e) => {
                self.error(&e);
            }
        }
    }
    fn publish_preview(&mut self, output: Output) {
        let count = output.images.len();
        let stamp = self.view().point.stamp;
        let state = &mut self.windows.selected_mut().preview;
        state.stamp = Some(stamp);
        state.images = output.images;
        let capability = if crate::terminal::graphics::supported() {
            "Kitty PNG output admitted"
        } else {
            "terminal PNG unavailable; org-open-at-point or file+sys: selects viewer"
        };
        let detail = output.details.first().map_or("", String::as_str);
        self.message(&format!(
            "Org previews: {count} PNG, {} fallback; {capability}; {detail}",
            output.fallback
        ));
    }
}
