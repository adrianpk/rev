//! Source input stays revision-qualified from selection through owned cleanup.
mod prompts;
use super::{Action, Pending, Session, outline::Request};
use crate::{
    command::{Command, OrgCommand as C},
    config::SourceProgram,
    org::{
        block::Definition,
        execution::{self, Outcome, State},
    },
};
pub(super) enum Prompt {
    Profile,
    Confirm,
    Inspect,
}
pub(super) struct Selection {
    request: Request,
    definition: Definition,
    profiles: Vec<SourceProgram>,
    work: Option<execution::Work>,
}
impl Session {
    /// Last bounded execution record; canonical RESULTS text is never implicit.
    pub fn execution_result(&self) -> Option<&Outcome> {
        self.execution_result.as_ref()
    }
    pub(super) fn execution_command(&mut self, command: Command) -> Option<Action> {
        if command == Command::KeyboardQuit
            && (self.execution.is_some() || matches!(self.pending, Some(Pending::Execution(_))))
        {
            return Some(self.cancel_execution());
        }
        if self.execution.is_some() {
            return if matches!(command, Command::Motion(_) | Command::Scroll(_)) {
                None
            } else {
                Some(self.message("Source execution owns input; C-g cancels"))
            };
        }
        if command == Command::Org(C::ExecutionResults) {
            return Some(self.inspect_execution());
        }
        None
    }
    pub(super) fn execution_current(&self) -> bool {
        self.execution.as_ref().is_some_and(|s| {
            self.windows.selected_id() == s.request.view
                && self
                    .buffer()
                    .is_ok_and(|b| b.stamp() == s.definition.stamp && b.is_org())
        })
    }
    pub(in crate::session) fn prepare_execution(
        &mut self,
        request: Request,
        definition: Definition,
    ) -> Result<(), String> {
        if self.execution.is_some() || self.pending.is_some() || self.windows.retained_view_active()
        {
            return Err("Another source interaction owns input".into());
        }
        let profiles: Vec<_> = self
            .buffers
            .snapshot()
            .settings()
            .org
            .source
            .programs
            .iter()
            .filter(|p| p.language == definition.language)
            .cloned()
            .collect();
        if profiles.is_empty() {
            return Err(format!(
                "No source program configured for {}; set org.source.programs",
                definition.language
            ));
        }
        self.execution = Some(Selection {
            request,
            definition,
            profiles,
            work: None,
        });
        let result = self.select_execution();
        if result.is_err() {
            self.execution = None;
        }
        result
    }
    fn select_execution(&mut self) -> Result<(), String> {
        let profiles = &self
            .execution
            .as_ref()
            .ok_or("No source selection")?
            .profiles;
        if profiles.len() == 1 {
            return self.choose_execution(&profiles[0].name.clone());
        }
        let names = profiles
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        self.input
            .argument(&format!("Source program ({names}): "), "")
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Execution(Prompt::Profile));
        self.message("Select a declared source program; C-g cancels");
        Ok(())
    }
    fn choose_execution(&mut self, name: &str) -> Result<(), String> {
        if !self.execution_current() {
            return Err("Source execution selection is stale".into());
        }
        let selection = self.execution.as_ref().ok_or("No source selection")?;
        let profile = selection
            .profiles
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .ok_or("Choose a declared program name")?;
        let work = execution::Work::new(
            self.buffer().map_err(|e| e.to_string())?,
            selection.definition.clone(),
            profile,
            self.buffers.snapshot(),
        )?;
        let confirm = self
            .buffers
            .snapshot()
            .settings()
            .org
            .source
            .confirm_execution
            || work.force_query();
        self.execution.as_mut().expect("selection").work = Some(work);
        if confirm {
            self.confirm_execution()
        } else {
            self.launch_execution()
        }
    }
    fn launch_execution(&mut self) -> Result<(), String> {
        if !self.execution_current() {
            return Err("Source changed before confirmed launch".into());
        }
        let selection = self.execution.as_mut().ok_or("No source selection")?;
        let work = selection.work.take().ok_or("No confirmed source program")?;
        self.org.start_execution(selection.definition.stamp, work)?;
        self.message("Source program running; C-g cancels and reaps owned work");
        Ok(())
    }
    pub(super) fn finish_execution(&mut self, result: Result<Outcome, String>) {
        let current = self.execution_current();
        self.execution = None;
        match result {
            Ok(mut out) => {
                if !current {
                    out.state = State::Stale;
                    out.detail =
                        "Source context changed; output retained without publication".into();
                }
                self.message(&format!(
                    "Source {:?}: {}; org-babel-execution-results inspects output",
                    out.state, out.detail
                ));
                self.execution_result = Some(out);
            }
            Err(e) => {
                self.error(&e);
            }
        }
    }
    fn cancel_execution(&mut self) -> Action {
        self.pending = None;
        self.input.cancel();
        if self.org.execution_running {
            self.org.cancel_execution();
            return self.message("Source cancellation requested; waiting for owned cleanup");
        }
        self.execution = None;
        self.message("Source execution cancelled before launch; source unchanged")
    }
}
