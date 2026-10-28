use super::*;
use crate::command::{Command, OrgCommand as C, Scroll};
impl Session {
    pub(in crate::session) fn habit_command(&mut self, command: Command) -> Option<Action> {
        self.habit_agenda.as_ref()?;
        match command {
            Command::KeyboardQuit => {
                if self.input.focus() != crate::input::Focus::Editing {
                    self.input.cancel();
                    return Some(self.message("Habit view prompt cancelled"));
                }
                self.habit_agenda = None;
                self.sync_input_context();
                Some(self.message("Returned to Org source"))
            }
            Command::Org(c) => self.habit_org(c),
            Command::Quit => None,
            Command::Scroll(m) => Some(self.habit_motion(m)),
            _ => Some(self.message("Read-only habit agenda; C-g returns to source")),
        }
    }
    fn habit_org(&mut self, command: C) -> Option<Action> {
        if command == C::AgendaRefresh {
            return Some(self.org_command(C::HabitGraph));
        }
        if command == C::HabitGraph || command.timer_control() {
            return None;
        }
        if command.habit() {
            return Some(self.habit_control(command));
        }
        Some(self.message("Read-only habit agenda; C-g returns to source"))
    }
    fn habit_control(&mut self, command: C) -> Action {
        if matches!(command, C::HabitDate | C::HabitWindow) {
            return self.habit_prompt(command);
        }
        let state = self.habit_agenda.as_mut().expect("active habit owner");
        match command {
            C::HabitToggle => state.shown = !state.shown,
            C::HabitTodayOnly => state.policy.today_only = !state.policy.today_only,
            C::HabitAllToday => state.policy.show_all_today = !state.policy.show_all_today,
            _ => unreachable!("qualified habit control"),
        }
        state.top = 0;
        self.message("Habit agenda display changed")
    }
    fn habit_motion(&mut self, motion: Scroll) -> Action {
        let state = self.habit_agenda.as_mut().expect("active habit owner");
        let visible = state
            .agenda
            .visible(state.date, &state.policy, state.shown)
            .count();
        match motion {
            Scroll::NextLine => {
                state.top = state.top.saturating_add(1).min(visible.saturating_sub(1))
            }
            Scroll::PreviousLine => state.top = state.top.saturating_sub(1),
            _ => (),
        }
        Action::Redraw
    }
}
