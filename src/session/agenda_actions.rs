//! Explicit source actions reuse ordinary handlers under a mapped saved view.
mod bulk;
mod execute;
mod marks;
mod resolve;
mod results;
use super::{Action, Session};
use crate::{
    buffer::{Stamp, edit::Point},
    command::{Command, OrgCommand as C},
    org::agenda::{ActionResult, ActionStatus, ActionTarget, CalendarLine},
    window::ViewId,
};
pub(in crate::session) use bulk::BulkPrompt;
pub(in crate::session) use marks::Marks;
use std::time::{Duration, Instant};
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Resolve,
    Waiting,
    Edit,
    Acting,
}
pub(super) struct Run {
    owner: ViewId,
    targets: Vec<ActionTarget>,
    pins: Vec<Option<usize>>,
    command: C,
    phase: Phase,
    index: usize,
    before: Option<Stamp>,
    deadline: Instant,
    outcomes: Vec<ActionResult>,
    argument: Option<String>,
    replayed: bool,
}
impl Session {
    pub fn agenda_action_results(&self) -> &[ActionResult] {
        &self.agenda_results
    }
    pub(super) fn agenda_action_command(&mut self, command: Command) -> Option<Action> {
        if self.agenda_run.is_some() {
            return self.running_agenda_command(command);
        }
        if let Some(action) = self.agenda_results_command(command) {
            return Some(action);
        }
        match command {
            Command::Org(c) if self.agenda_active() && c.agenda_bulk() => {
                Some(self.agenda_bulk_command(c))
            }
            Command::Org(c) if self.agenda_active() && c.agenda_source_action() => {
                Some(self.begin_agenda_action(c))
            }
            _ => None,
        }
    }
    fn running_agenda_command(&mut self, command: Command) -> Option<Action> {
        match command {
            Command::KeyboardQuit => {
                self.cancel_agenda_action();
                Some(Action::Redraw)
            }
            Command::Window(_) | Command::Quit | Command::ExecuteExtended => {
                self.cancel_agenda_action();
                None
            }
            _ if self.pending.is_some() => None,
            _ => Some(self.message("Agenda source action pending; C-g cancels")),
        }
    }
    fn selected_agenda_target(&self) -> Result<ActionTarget, String> {
        if let Some(s) = &self.query_agenda
            && self.query_active()
        {
            let row = s
                .result
                .row_at(s.cursor)
                .ok_or("Select an agenda entry, not a section or failure")?;
            return Ok(ActionTarget {
                source: s.result.sources[row.source].clone(),
                byte: row.byte,
                title: row.title.clone(),
            });
        }
        let s = self
            .calendar_agenda
            .as_ref()
            .ok_or("Open an agenda first")?;
        let Some(CalendarLine::Entry(index)) = s.calendar.lines.get(s.cursor) else {
            return Err("Select an agenda entry, not a section or failure".into());
        };
        let row = &s.calendar.rows[*index];
        Ok(ActionTarget {
            source: s.calendar.sources[row.source].clone(),
            byte: row.byte,
            title: row.title.clone(),
        })
    }
    fn begin_agenda_action(&mut self, command: C) -> Action {
        if self.org.busy() {
            return self.message("Org observation pending; C-g cancels");
        }
        let target = match self.selected_agenda_target() {
            Ok(target) => target,
            Err(e) => return self.error(&e),
        };
        self.start_agenda_targets(vec![target], command, None)
    }
    fn start_agenda_targets(
        &mut self,
        targets: Vec<ActionTarget>,
        command: C,
        argument: Option<String>,
    ) -> Action {
        if targets.is_empty() || targets.len() > 256 {
            return self.error("Select 1..256 distinct source targets");
        }
        self.agenda_results.clear();
        self.windows.clear_source_anchors();
        self.agenda_run = Some(Run {
            owner: self.windows.selected_id(),
            pins: vec![None; targets.len()],
            targets,
            command,
            phase: Phase::Resolve,
            index: 0,
            before: None,
            deadline: Instant::now()
                + Duration::from_secs(
                    self.buffers.snapshot().settings().org.limits.job_seconds as u64,
                ),
            outcomes: Vec::new(),
            argument,
            replayed: false,
        });
        self.advance_agenda_action();
        self.message("Checking agenda source targets; C-g cancels")
    }
    pub(super) fn agenda_action_remaining(&self) -> Option<Duration> {
        let run = self.agenda_run.as_ref()?;
        if self.org.busy() {
            return None;
        }
        if self.org.request.is_some() {
            return Some(run.deadline.saturating_duration_since(Instant::now()));
        }
        if run.argument.is_some() && !run.replayed && bulk::primary(self.pending.as_ref()) {
            return Some(Duration::ZERO);
        }
        if run.phase == Phase::Waiting || self.pending.is_some() {
            return None;
        }
        Some(Duration::ZERO)
    }
    pub(super) fn expire_agenda_action(&mut self) -> Action {
        if self.agenda_action_remaining() != Some(Duration::ZERO) {
            return Action::Wait;
        }
        self.advance_agenda_action();
        Action::Redraw
    }
    pub(super) fn advance_agenda_action(&mut self) {
        let Some(mut run) = self.agenda_run.take() else {
            return;
        };
        if self.windows.selected_id() != run.owner {
            self.agenda_run = Some(run);
            self.cancel_agenda_action();
            return;
        }
        match run.phase {
            Phase::Resolve => self.resolve_agenda_target(&mut run),
            Phase::Edit => self.edit_agenda_target(&mut run),
            Phase::Acting => self.finish_agenda_target(&mut run),
            Phase::Waiting => (),
        }
        if run.phase == Phase::Edit && run.index == run.targets.len() {
            self.complete_agenda_actions(run);
        } else {
            self.agenda_run = Some(run);
        }
    }
    fn cancel_agenda_action(&mut self) {
        self.org.cancel();
        self.input.cancel();
        self.pending = None;
        if let Some(mut run) = self.agenda_run.take() {
            self.retain_cancelled_agenda_change(&mut run);
            for index in 0..run.targets.len() {
                if !run
                    .outcomes
                    .iter()
                    .any(|r| r.target.same(&run.targets[index]))
                {
                    run.outcome(
                        index,
                        ActionStatus::Cancelled,
                        "Cancelled before this target completed".into(),
                        None,
                    );
                }
            }
            self.complete_agenda_actions(run);
        }
    }
    fn retain_cancelled_agenda_change(&self, run: &mut Run) {
        let Some(before) = run.before else {
            return;
        };
        let after = self.buffers.get(before.buffer).map(|b| b.stamp()).ok();
        if after.is_some_and(|stamp| stamp != before) {
            run.outcome(run.index,ActionStatus::ChangedWithFailure,"Source changed before cancellation; inspect canonical history; no rollback claimed".into(),after);
        }
    }
    fn complete_agenda_actions(&mut self, mut run: Run) {
        self.windows.restore_source_action();
        self.windows.clear_source_anchors();
        run.outcomes.sort_by_key(|outcome| {
            run.targets
                .iter()
                .position(|t| t.same(&outcome.target))
                .unwrap_or(usize::MAX)
        });
        self.agenda_results = run.outcomes;
        if run.argument.is_some() {
            self.clear_agenda_marks();
            self.agenda_results_view = Some((run.owner, 0));
        }
        self.sync_input_context();
        let changed = self
            .agenda_results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    ActionStatus::Changed | ActionStatus::ChangedWithFailure
                )
            })
            .count();
        if let [result] = self.agenda_results.as_slice() {
            let feedback = format!("Agenda action {:?}: {}", result.status, result.message);
            self.message(&feedback);
        } else {
            self.message(&format!(
                "Agenda actions: {changed} changed, {} other outcomes; sources retained in buffers",
                self.agenda_results.len() - changed
            ));
        }
    }
}
impl Run {
    fn outcome(
        &mut self,
        index: usize,
        status: ActionStatus,
        message: String,
        after: Option<Stamp>,
    ) {
        self.outcomes.push(ActionResult {
            target: self.targets[index].clone(),
            status,
            message,
            after,
        });
    }
}
