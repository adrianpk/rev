//! One checked native refile interaction and the last recoverable move result.
mod publish;
mod results;
mod selection;
#[cfg(test)]
mod tests;
use super::{Action, Pending, Session, outline::Request};
use crate::{
    command::{Command, OrgCommand as C},
    input::Focus,
    org::{
        Outline,
        agenda::{SourceScan, SourceSelection},
        refile::{Plan, Subtree, Targets},
    },
};
pub use results::{ResultEntry, Status};
use std::sync::Arc;
pub(super) struct State {
    request: Request,
    subtree: Subtree,
    copy: bool,
    targets: Option<Targets>,
}
impl Session {
    pub(in crate::session) fn start_refile(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        let subtree = self.refile_subtree(&model, &request)?;
        let copy = request.command == C::RefileCopy;
        let selection = self.refile_selection()?;
        let limits = &self.buffers.snapshot().settings().org.limits;
        let scan = SourceScan::new(&self.buffers, selection, limits)?;
        self.org.start_refile_scan(
            subtree.stamp(),
            scan,
            limits.agenda_rows,
            limits.job_seconds,
        )?;
        self.refile = Some(State {
            request,
            subtree,
            copy,
            targets: None,
        });
        self.message("Refile targets pending; source unchanged; C-g cancels");
        Ok(())
    }
    fn refile_subtree(&self, model: &Outline, request: &Request) -> Result<Subtree, String> {
        if self.capture.is_some() {
            return Err("Capture owns its draft; finalize or abort before refiling".into());
        }
        if self.refile.is_some() {
            return Err("Refile pending; C-g cancels".into());
        }
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if buffer.is_transient() {
            return Err("Transient drafts cannot own a refile source".into());
        }
        let span = model.refile_span(buffer, self.view().point.position)?;
        let copy = request.command == C::RefileCopy;
        if !copy && buffer.is_readonly() {
            return Err("Refile source is read-only".into());
        }
        Subtree::checked(buffer, span)
    }
    pub(in crate::session) fn finish_refile_targets(&mut self, result: Result<Targets, String>) {
        let result = result.and_then(|targets| {
            let state = self.refile.as_mut().ok_or("Refile was cancelled")?;
            if targets.headings.is_empty() {
                return Err("No eligible refile headings in selected sources".into());
            }
            state.targets = Some(targets);
            Ok(())
        });
        if let Err(error) = result {
            self.fail_refile(&error);
            return;
        }
        if let Err(error) = self.prompt_refile("") {
            self.fail_refile(&error);
        }
    }
    pub(in crate::session) fn finish_refile_plan(&mut self, result: Result<Plan, String>) {
        let Some(state) = self.refile.take() else {
            return;
        };
        let result = result.and_then(|plan| {
            if !self.org_source_current(&state.request)
                || self.input.focus() != Focus::Editing
                || self.pending.is_some()
            {
                return Err("Refile source or view changed; nothing moved".into());
            }
            self.publish_refile(plan)
        });
        match result {
            Ok(()) => {
                self.show_refile_results();
            }
            Err(error) => self.fail_refile(&error),
        }
    }
    pub(in crate::session) fn fail_refile(&mut self, error: &str) {
        self.refile = None;
        if matches!(self.pending, Some(Pending::Refile)) {
            self.pending = None;
            self.input.cancel();
        }
        self.error(&format!("Refile rejected; nothing moved: {error}"));
    }
    pub(in crate::session) fn cancel_refile(&mut self, command: Command) -> Option<Action> {
        if command != Command::KeyboardQuit || self.refile.is_none() {
            return None;
        }
        self.org.cancel();
        self.refile = None;
        if matches!(self.pending, Some(Pending::Refile)) {
            self.pending = None;
            self.input.cancel();
        }
        Some(self.message("Refile cancelled; source and destination unchanged"))
    }
}
