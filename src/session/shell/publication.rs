use super::Request;
use crate::{buffer::edit::Replacement, session::Session, shell::Outcome};

impl Session {
    pub(super) fn publish_shell(
        &mut self,
        request: &Request,
        outcome: &Outcome,
    ) -> Result<bool, String> {
        let target = &request.target;
        let id = target.source.stamp.buffer;
        if self.observation_pending_for(id) || self.reload_pending_for(id) {
            return Err(
                "Shell target has pending external observation or reload; use revert-buffer".into(),
            );
        }
        if let Some(file) = &target.file {
            let observed = outcome.observed.as_ref().map_err(String::clone)?;
            if observed.as_ref() != file.observation() {
                return Err("Shell target has external file changes; use revert-buffer".into());
            }
        }
        let buffer = self
            .buffers
            .get_mut(id)
            .map_err(|e| format!("Shell target was closed: {e}"))?;
        target.current(buffer)?;
        if target.source.range.is_empty() && outcome.stdout.is_empty() {
            return Ok(false);
        }
        self.windows
            .publish_observed(
                buffer,
                Replacement {
                    source: target.source.clone(),
                    text: outcome.stdout.clone(),
                },
                request.home.clone(),
                request.view,
            )
            .map_err(|e| e.to_string())?;
        buffer.deactivate_region();
        self.invalidate_completion();
        self.published_repeat_boundary();
        self.activate_lsp();
        self.observe_files();
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
