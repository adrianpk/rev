//! One immutable tangle plan, explicit per-file consent and retained outcomes.
mod lifecycle;
use super::{Action, Pending, Session, outline::Request};
use crate::{
    command::{Command, OrgCommand as C},
    coordinates::Coordinates,
    file::Publication,
    org::tangle::{self, JobWork, Output, Prepared, PreparedPlan, Selection},
};
use std::path::PathBuf;
pub(super) enum Prompt {
    Replace(Box<Prepared>),
    Result,
}
pub(super) struct State {
    request: Request,
    plan: Option<PreparedPlan>,
    active: Option<PathBuf>,
    cancelled: bool,
}
impl Session {
    pub fn tangling_results(&self) -> &[tangle::Outcome] {
        &self.tangling_results
    }
    pub(super) fn tangle_command(&mut self, command: Command) -> Option<Action> {
        if command == Command::KeyboardQuit
            && (self.tangling.is_some() || matches!(self.pending, Some(Pending::Tangle(_))))
        {
            return Some(self.cancel_tangle());
        }
        if self.tangling.is_some() {
            return self.tangle_active_command(command);
        }
        match command {
            Command::Org(C::Tangle | C::TangleBlock) => Some(match self.begin_tangle(command) {
                Ok(()) => Action::Redraw,
                Err(e) => {
                    if self.tangling.is_some() {
                        self.end_tangle(&e);
                    }
                    self.error(&e)
                }
            }),
            Command::Org(C::TangleResults) => Some(self.tangle_result_prompt()),
            _ => None,
        }
    }
    pub(super) fn tangle_input_blocked(&self, outcome: &crate::input::Outcome) -> bool {
        (self.tangling.is_some() || self.execution.is_some())
            && matches!(
                outcome,
                crate::input::Outcome::Pointer(_) | crate::input::Outcome::Editing(_)
            )
    }
    fn tangle_active_command(&mut self, command: Command) -> Option<Action> {
        if matches!(command, Command::Motion(_) | Command::Scroll(_)) {
            None
        } else {
            Some(self.message("Tangling owns source input; C-g cancels remaining files"))
        }
    }
    fn begin_tangle(&mut self, command: Command) -> Result<(), String> {
        self.admit_tangle()?;
        self.stop_typing().map_err(|e| e.to_string())?;
        let b = self.buffer().map_err(|e| e.to_string())?;
        let at = Coordinates::new(b)
            .map_err(|e| e.to_string())?
            .to_byte(self.view().point.position)
            .map_err(|e| e.to_string())?;
        let selection = if command == Command::Org(C::TangleBlock) {
            Selection::Block(at)
        } else {
            Selection::Buffer
        };
        let work = tangle::Work::new(b, selection)?;
        let request = Request {
            view: self.windows.selected_id(),
            origin: self.view().clone(),
            command: match command {
                Command::Org(c) => c,
                _ => unreachable!("tangle command"),
            },
        };
        let inputs = self
            .buffers
            .iter()
            .filter_map(|b| b.file().map(|f| f.path().to_owned()))
            .collect();
        self.tangling_results.clear();
        self.tangling = Some(State {
            request,
            plan: None,
            active: None,
            cancelled: false,
        });
        self.launch_tangle(JobWork::Prepare(work, inputs))
    }
    fn admit_tangle(&self) -> Result<(), String> {
        if self.org.busy() || self.pending.is_some() || self.windows.retained_view_active() {
            return Err("Another Org interaction owns source input".into());
        }
        if self.input.focus() != crate::input::Focus::Editing
            || self.tree.focused()
            || self.agenda_active()
            || self.habit_agenda.is_some()
        {
            return Err("Org editing owns tangling".into());
        }
        if !self.buffer().is_ok_and(|b| b.is_org()) {
            return Err("Native Org mode is not active".into());
        }
        Ok(())
    }
    fn launch_tangle(&mut self, work: JobWork) -> Result<(), String> {
        let stamp = self
            .tangling
            .as_ref()
            .ok_or("No tangle selection")?
            .request
            .origin
            .point
            .stamp;
        let seconds = self.buffers.snapshot().settings().org.limits.job_seconds;
        self.org.start_tangle(stamp, work, seconds)?;
        self.message("Preparing or publishing tangle files; C-g cancels remaining work");
        Ok(())
    }
    pub(super) fn tangle_current(&self) -> bool {
        self.tangling.as_ref().is_some_and(|s| {
            !s.cancelled
                && self.windows.selected_id() == s.request.view
                && self
                    .buffer()
                    .is_ok_and(|b| b.is_org() && b.stamp() == s.request.origin.point.stamp)
        })
    }
    pub(super) fn finish_tangle(&mut self, result: Result<Output, String>) {
        let result = self.tangle_output(result).and_then(|()| self.next_tangle());
        if let Err(e) = result {
            self.end_tangle(&e);
        }
    }
    fn tangle_output(&mut self, result: Result<Output, String>) -> Result<(), String> {
        match result? {
            Output::Plan(plan) => {
                let b = self.buffer().map_err(|e| e.to_string())?;
                if plan.stamp != b.stamp() {
                    return Err("Tangle source changed before publication".into());
                }
                for file in &plan.files {
                    self.export_target_available(&file.target)?;
                }
                if let Some(state) = &mut self.tangling {
                    state.plan = Some(plan);
                }
            }
            Output::File(outcome) => {
                self.tangling_results.push(outcome);
                if let Some(state) = &mut self.tangling {
                    state.active = None;
                }
            }
        }
        Ok(())
    }
    fn next_tangle(&mut self) -> Result<(), String> {
        if !self.tangle_current() {
            self.end_tangle("Tangle cancelled or source changed");
            return Ok(());
        }
        let next = self
            .tangling
            .as_mut()
            .and_then(|s| s.plan.as_mut())
            .and_then(|p| p.files.pop_front());
        match next {
            Some(file) => self.review_tangle(file),
            None => {
                self.tangling = None;
                self.tangle_summary("Tangle finished");
                Ok(())
            }
        }
    }
    fn review_tangle(&mut self, file: Prepared) -> Result<(), String> {
        if let Some(state) = &mut self.tangling {
            state.active = Some(file.target.path().into());
        }
        self.export_target_available(&file.target)?;
        if !file.target.is_present() {
            return self.write_tangle(file);
        }
        let label = format!(
            "Replace tangle output {}? yes/no: ",
            file.target.path().display()
        );
        self.input.argument(&label, "").map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Tangle(Prompt::Replace(Box::new(file))));
        Ok(())
    }
    fn write_tangle(&mut self, file: Prepared) -> Result<(), String> {
        if !self.tangle_current() {
            return Err("Tangle source changed before file publication".into());
        }
        self.export_target_available(&file.target)?;
        let work = file.write(self.buffer().map_err(|e| e.to_string())?);
        self.launch_tangle(JobWork::Write(Box::new(work)))
    }
    pub(super) fn tangle_argument(&mut self, prompt: Prompt, value: &str) -> Action {
        if let Prompt::Result = prompt {
            return self.tangle_result(value);
        }
        let Prompt::Replace(file) = prompt else {
            unreachable!("result handled")
        };
        let result = match value.trim() {
            "yes" => self.write_tangle(*file),
            "no" => {
                self.tangling_results
                    .push(file.failed("Replacement declined"));
                if let Some(s) = &mut self.tangling {
                    s.active = None;
                }
                self.next_tangle()
            }
            _ => self.review_tangle(*file),
        };
        match result {
            Ok(()) => Action::Redraw,
            Err(e) => {
                self.end_tangle(&e);
                Action::Redraw
            }
        }
    }
}
