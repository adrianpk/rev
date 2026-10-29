//! One explicit project selection, serial checked files and retained outcomes.
mod arguments;
mod results;
use super::{Action, Pending, Session, outline::Request};
use crate::{
    command::{Command, OrgCommand},
    file::Publication,
    org::publish::{self, Item, JobWork, Output, Plan, Prepared, Selection},
};
use std::path::PathBuf;
pub(super) enum Prompt {
    Select(Request),
    Replace(Box<Prepared>),
    Upload(publish::Upload),
    Result,
}
pub(super) struct State {
    request: Request,
    plan: Option<Plan>,
    inputs: Vec<PathBuf>,
    active: Option<(PathBuf, PathBuf)>,
    cancelled: bool,
    evidence: Vec<crate::org::export::Evidence>,
}
impl Session {
    pub(super) fn publishing_command(&mut self, command: OrgCommand) -> Option<Action> {
        if command == OrgCommand::PublishResults {
            return Some(self.publishing_result_prompt());
        }
        if command != OrgCommand::Publish {
            return None;
        }
        Some(match self.begin_publishing(command) {
            Ok(()) => Action::Redraw,
            Err(e) => self.error(&e),
        })
    }
    fn begin_publishing(&mut self, command: OrgCommand) -> Result<(), String> {
        if self.org.busy() || self.publishing.is_some() || self.pending.is_some() {
            return Err("Org interaction pending; C-g cancels".into());
        }
        let projects = &self.buffers.snapshot().settings().org.publish.projects;
        if projects.is_empty() {
            return Err("No projects configured; set org.publish.projects".into());
        }
        let names = projects
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let request = Request {
            view: self.windows.selected_id(),
            origin: self.view().clone(),
            command,
        };
        self.publishing_prompt(
            Prompt::Select(request),
            &format!("Publish project ({names}): "),
        )
    }
    fn publishing_prompt(&mut self, prompt: Prompt, label: &str) -> Result<(), String> {
        self.input.argument(label, "").map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Publish(Box::new(prompt)));
        self.message("Select the publishing operation; C-g cancels");
        Ok(())
    }
    fn launch_publishing(&mut self, work: JobWork) -> Result<(), String> {
        let state = self
            .publishing
            .as_ref()
            .ok_or("Project selection no longer active")?;
        let seconds = self.buffers.snapshot().settings().org.limits.job_seconds;
        self.org
            .start_publishing(state.request.origin.point.stamp, work, seconds)?;
        self.message("Org project publishing; C-g cancels remaining files");
        Ok(())
    }
    pub(super) fn publishing_current(&self) -> bool {
        let Some(state) = &self.publishing else {
            return false;
        };
        if state.cancelled {
            return false;
        }
        let request = &state.request;
        self.windows.selected_id() == request.view
            && self
                .buffers
                .get(request.origin.point.stamp.buffer)
                .is_ok_and(|b| b.stamp() == request.origin.point.stamp)
            && state
                .plan
                .as_ref()
                .is_none_or(|p| p.current(&self.buffers).is_ok())
    }
    pub(super) fn finish_publishing(&mut self, result: Result<Output, String>) {
        if let Err(e) = self
            .publishing_output(result)
            .and_then(|()| self.next_publication())
        {
            self.end_publishing(&e);
        }
    }
    fn publishing_output(&mut self, result: Result<Output, String>) -> Result<(), String> {
        match result? {
            Output::Plan(plan) => {
                let inputs = plan.inputs()?;
                let state = self
                    .publishing
                    .as_mut()
                    .ok_or("Project selection cancelled")?;
                state.inputs = inputs;
                state.plan = Some(plan);
            }
            Output::File(outcome) => {
                self.publishing_results.push(outcome);
                if let Some(state) = &mut self.publishing {
                    state.active = None;
                }
            }
            Output::Uploaded(result) => {
                let message = result.unwrap_or_else(|e| e);
                self.publishing = None;
                self.message(&message);
            }
        }
        Ok(())
    }
    fn next_publication(&mut self) -> Result<(), String> {
        if self.publishing.is_none() {
            return Ok(());
        }
        if !self.publishing_current() {
            self.end_publishing("Project cancelled or source changed");
            return Ok(());
        }
        while let Some(item) = self
            .publishing
            .as_mut()
            .and_then(|s| s.plan.as_mut())
            .and_then(|p| p.items.pop_front())
        {
            match item {
                Item::Issue(issue) => self.publishing_results.push(issue),
                Item::Ready(prepared) => {
                    if let Err(e) = self
                        .export_target_available(&prepared.target)
                        .and_then(|()| prepared.current(&self.buffers))
                    {
                        self.publishing_results.push(prepared.failed(&e));
                        continue;
                    }
                    return self.review_publication(prepared);
                }
            }
        }
        self.publishing_complete()
    }
    fn review_publication(&mut self, prepared: Box<Prepared>) -> Result<(), String> {
        let state = self
            .publishing
            .as_mut()
            .ok_or("Project selection cancelled")?;
        state.active = Some((prepared.source.clone(), prepared.target.path().into()));
        state.evidence = prepared.evidence();
        if prepared.target.is_present() {
            let label = format!(
                "Replace project output {}? yes/no: ",
                prepared.target.path().display()
            );
            return self.publishing_prompt(Prompt::Replace(prepared), &label);
        }
        self.write_publication(*prepared)
    }
    fn write_publication(&mut self, prepared: Prepared) -> Result<(), String> {
        prepared.current(&self.buffers)?;
        self.export_target_available(&prepared.target)?;
        let inputs = self
            .publishing
            .as_ref()
            .ok_or("Project selection cancelled")?
            .inputs
            .clone();
        let active = (prepared.source.clone(), prepared.target.path().into());
        let work = prepared.write(self.buffer().map_err(|e| e.to_string())?, inputs);
        self.publishing.as_mut().expect("selected project").active = Some(active);
        self.launch_publishing(JobWork::Write(Box::new(work)))
    }
}
