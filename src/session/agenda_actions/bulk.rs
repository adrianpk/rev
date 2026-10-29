use super::*;
use crate::session::Pending;
#[derive(Clone, Copy)]
pub(in crate::session) enum BulkPrompt {
    Action,
    Value(C),
}
impl Session {
    pub(super) fn prompt_agenda_bulk(&mut self, stage: BulkPrompt, draft: &str) -> Action {
        if self.agenda_marks.targets.is_empty()
            || self.agenda_marks.owner != Some(self.windows.selected_id())
        {
            return self.error("Mark agenda source targets before a bulk action");
        }
        let label = match stage {
            BulkPrompt::Action => {
                "Bulk action [todo/tags/remove-tag/schedule/deadline/priority/effort]: "
            }
            BulkPrompt::Value(_) => "Bulk value [ordinary source grammar]: ",
        };
        if let Err(e) = self.input.argument(label, draft) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::AgendaBulk(stage));
        self.message(&format!(
            "{} explicit source targets; C-g cancels before source edits",
            self.agenda_marks.targets.len()
        ))
    }
    pub(in crate::session) fn agenda_bulk_argument(
        &mut self,
        stage: BulkPrompt,
        draft: &str,
    ) -> Action {
        match stage {
            BulkPrompt::Action => match operation(draft) {
                Ok(command) => self.prompt_agenda_bulk(BulkPrompt::Value(command), ""),
                Err(e) => {
                    self.prompt_agenda_bulk(stage, draft);
                    self.error(&e)
                }
            },
            BulkPrompt::Value(command) => {
                if self.agenda_marks.owner != Some(self.windows.selected_id()) {
                    return self.error("Agenda selection owner changed; select again");
                }
                self.start_agenda_targets(
                    self.agenda_marks.targets.clone(),
                    command,
                    Some(draft.into()),
                )
            }
        }
    }
    pub(super) fn replay_agenda_argument(&mut self, run: &mut Run) -> bool {
        if run.replayed || run.argument.is_none() || !primary(self.pending.as_ref()) {
            return false;
        }
        run.replayed = true;
        let draft = run.argument.as_ref().expect("bulk value");
        let draft = match (&self.pending, run.command) {
            (Some(Pending::Tags(prompt)), C::SetTags) => prompt.agenda_append(draft),
            _ => Ok(draft.clone()),
        };
        self.input.cancel();
        match draft {
            Ok(draft) => {
                self.argument(draft);
            }
            Err(error) => {
                self.pending = None;
                self.error(&error);
                return true;
            }
        }
        if primary(self.pending.as_ref()) {
            let feedback = self.echo.clone();
            self.input.cancel();
            self.pending = None;
            self.error(&format!("Bulk value rejected for this target: {feedback}"));
        }
        true
    }
}
pub(super) fn primary(pending: Option<&Pending>) -> bool {
    match pending {
        Some(Pending::Task(p)) => p.agenda_primary(),
        Some(Pending::Timestamp(p)) => p.agenda_primary(),
        Some(Pending::Estimate(_) | Pending::Tags(_)) => true,
        _ => false,
    }
}
fn operation(draft: &str) -> Result<C, String> {
    let choices = [
        (["todo", "t"], C::SetTodo),
        (["tags", "+"], C::SetTags),
        (["remove-tag", "-"], C::RemoveTag),
        (["schedule", "s"], C::Schedule),
        (["deadline", "d"], C::Deadline),
        (["priority", ","], C::Priority),
        (["effort", "e"], C::SetEffort),
    ];
    choices
        .iter()
        .find(|(names, _)| names.contains(&draft.trim()))
        .map(|(_, command)| *command)
        .ok_or_else(|| {
            "Select todo, tags, remove-tag, schedule, deadline, priority or effort".into()
        })
}
