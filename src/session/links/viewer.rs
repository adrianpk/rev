use super::*;
use crate::org::{
    io::process::Outcome,
    links::{External, Viewer},
};
pub(in crate::session) struct Prompt {
    request: Request,
    target: Follow,
    viewer: Viewer,
}
impl Session {
    pub(super) fn prompt_viewer(&mut self, target: Follow, request: Request) -> Result<(), String> {
        let viewer = Viewer::configured(self.buffers.snapshot())?;
        self.viewer_prompt(
            Prompt {
                request,
                target,
                viewer,
            },
            "",
        )
    }
    fn viewer_prompt(&mut self, prompt: Prompt, feedback: &str) -> Result<(), String> {
        self.input
            .argument("Run configured viewer? yes/no: ", "")
            .map_err(|e| e.to_string())?;
        self.message(&format!(
            "{} for {}; {feedback}",
            prompt.viewer.label(),
            prompt.target.label()
        ));
        self.pending = Some(Pending::Link(Box::new(super::Prompt::Viewer(prompt))));
        Ok(())
    }
    pub(super) fn viewer_argument(&mut self, prompt: Prompt, draft: &str) -> Action {
        if !self.org_source_current(&prompt.request) {
            return self.error("External link source or view changed; nothing launched");
        }
        match draft.trim() {
            "no" => self.message("External link cancelled; nothing launched"),
            "yes" => match self.launch_viewer(prompt) {
                Ok(()) => self.message("Configured viewer running; C-g cancels"),
                Err(error) => self.error(&error),
            },
            _ => match self.viewer_prompt(prompt, "Enter yes or no; C-g cancels") {
                Ok(()) => Action::Redraw,
                Err(error) => self.error(&error),
            },
        }
    }
    fn launch_viewer(&mut self, prompt: Prompt) -> Result<(), String> {
        let policy = &self.buffers.snapshot().settings().org;
        let snapshot = Snapshot::new(
            self.buffer().map_err(|e| e.to_string())?,
            prompt.request.origin.point,
            policy,
        )?;
        let work = External::new(snapshot, prompt.target, prompt.viewer);
        self.org
            .start_link_viewer(work, policy.limits.job_seconds)?;
        self.links.request = Some(prompt.request);
        Ok(())
    }
    pub(in crate::session) fn finish_link_viewer(&mut self, result: Result<Outcome, String>) {
        let Some(request) = self.links.request.take() else {
            return;
        };
        if !self.org_source_current(&request) {
            self.error("External link source or view changed; old result discarded");
            return;
        }
        match result {
            Ok(outcome) => {
                let output = outcome
                    .stdout
                    .iter()
                    .chain(&outcome.stderr)
                    .copied()
                    .collect::<Vec<_>>();
                let preview = String::from_utf8_lossy(&output)
                    .chars()
                    .take(512)
                    .flat_map(char::escape_default)
                    .collect::<String>();
                if outcome.status.success() {
                    self.message(&format!("Viewer completed: {}; {preview}", outcome.status));
                } else {
                    self.error(&format!("Viewer failed: {}; {preview}", outcome.status));
                }
            }
            Err(error) => {
                self.error(&error);
            }
        }
    }
}
