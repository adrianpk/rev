mod control;
mod frame;
mod prompt;
use super::*;
use crate::org::agenda::HabitAgenda;
use crate::org::time::Date;
pub(super) use prompt::Prompt;

pub(super) struct State {
    agenda: HabitAgenda,
    policy: crate::config::Habit,
    date: Date,
    shown: bool,
    top: usize,
    view: crate::window::ViewId,
}
impl Session {
    pub(super) fn start_habit_agenda(
        &mut self,
        model: std::sync::Arc<crate::org::Outline>,
        request: outline::Request,
    ) -> Result<(), String> {
        let mut policy = self.buffers.snapshot().settings().org.clone();
        if let Some(state) = &self.habit_agenda {
            policy.habit = state.policy.clone()
        }
        let operation = self.org_clock.unwrap_or(outline::timestamps::capture)(&policy.time)?;
        let today = crate::org::time::OrgDay::new(policy.time.extend_today_until as u8)?
            .date_at(operation.now())?;
        let scan = model.habit_scan(self.buffer().map_err(|e| e.to_string())?, today, &policy)?;
        self.org
            .start_report(request, outline::reports::Work::Habits(scan), operation)?;
        self.message("Org habit graphs pending; C-g cancels");
        Ok(())
    }
    pub(super) fn install_habit_agenda(&mut self, agenda: HabitAgenda) {
        let old = self.habit_agenda.take();
        let state = State {
            date: old.as_ref().map_or(agenda.today, |s| {
                if s.date == s.agenda.today {
                    agenda.today
                } else {
                    s.date
                }
            }),
            policy: old.as_ref().map_or_else(
                || self.buffers.snapshot().settings().org.habit.clone(),
                |s| s.policy.clone(),
            ),
            shown: old.as_ref().is_none_or(|s| s.shown),
            top: old.as_ref().map_or(0, |s| s.top),
            view: self.windows.selected_id(),
            agenda,
        };
        self.calendar_agenda = None;
        self.query_agenda = None;
        self.habit_agenda = Some(state);
        self.sync_input_context();
        self.message("Habit agenda: K visibility; A all today; C-u K today-only; d date; G graph; g refresh; C-g source");
    }
    pub(super) fn habit_paste(&mut self, text: &str) -> Action {
        if (self.habit_agenda.is_some() || self.agenda_active())
            && self.input.focus() == crate::input::Focus::Editing
        {
            return self.message("Read-only Org agenda; C-g returns to source");
        }
        self.paste_text(text)
    }
}
