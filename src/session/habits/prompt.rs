use super::*;
use crate::command::OrgCommand as C;
pub(in crate::session) struct Prompt {
    command: C,
    source: crate::buffer::Stamp,
}
impl Session {
    pub(super) fn habit_prompt(&mut self, command: C) -> Action {
        self.habit_prompt_value(command, "")
    }
    fn habit_prompt_value(&mut self, command: C, draft: &str) -> Action {
        let label = if command == C::HabitDate {
            "Habit agenda date YYYY-MM-DD: "
        } else {
            "Habit graph preceding following column: "
        };
        match self.input.argument(label, draft) {
            Ok(()) => {
                self.pending = Some(Pending::Habit(Prompt {
                    command,
                    source: self.view().point.stamp,
                }));
                self.message("Explicit habit view data; C-g cancels")
            }
            Err(error) => self.error(&error.to_string()),
        }
    }
    pub(in crate::session) fn habit_argument(&mut self, prompt: Prompt, draft: &str) -> Action {
        if self.view().point.stamp != prompt.source || self.habit_agenda.is_none() {
            return self.message("Habit source changed; request cancelled");
        }
        let result = self.habit_argument_value(prompt.command, draft);
        match result {
            Ok(()) => self.message("Habit agenda display changed"),
            Err(error) => {
                let action = self.habit_prompt_value(prompt.command, draft);
                self.message(&error);
                action
            }
        }
    }
    fn habit_argument_value(&mut self, command: C, draft: &str) -> Result<(), String> {
        if draft.len() > 128 {
            return Err("Habit view input exceeds 128 bytes".into());
        }
        let state = self
            .habit_agenda
            .as_mut()
            .ok_or("No habit agenda is active")?;
        if command == C::HabitDate {
            state.date = if draft.trim() == "today" {
                state.agenda.today
            } else {
                habit_date(draft.trim())?
            };
        } else {
            let values = draft
                .split_whitespace()
                .take(4)
                .map(str::parse::<usize>)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "Graph needs three unsigned integers")?;
            let [preceding, following, column] = values.as_slice() else {
                return Err("Graph needs preceding following column".into());
            };
            let mut policy = state.policy.clone();
            policy.preceding_days = *preceding;
            policy.following_days = *following;
            policy.graph_column = *column;
            state.agenda.redraw(&policy)?;
            state.policy = policy;
        }
        state.top = 0;
        Ok(())
    }
}

fn habit_date(text: &str) -> Result<Date, String> {
    if text.len() != 10 {
        return Err("Habit date requires YYYY-MM-DD or today".into());
    }
    crate::org::time::Timestamp::parse(&format!("[{text}]")).map(|s| s.date())
}

impl Session {
    pub(in crate::session) fn cancel_habit_prompt(
        &mut self,
        command: crate::command::Command,
    ) -> Option<Action> {
        if command != crate::command::Command::KeyboardQuit || self.habit_agenda.is_none() {
            return None;
        }
        if !matches!(
            self.pending,
            Some(Pending::Habit(_) | Pending::Timer(_) | Pending::Command)
        ) {
            return None;
        }
        self.pending = None;
        self.org.cancel();
        self.input.cancel();
        Some(self.message("Habit view prompt cancelled"))
    }
}
