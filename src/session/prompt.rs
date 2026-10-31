use crate::command::Command;
use crate::input::Focus;

use super::{Action, Pending, Session};

impl Session {
    pub(super) fn prepare_command(&mut self, command: Command) -> Option<Action> {
        if let Some(action) = self
            .cancel_publishing(command)
            .or_else(|| self.cancel_export(command))
            .or_else(|| self.cancel_table(command))
            .or_else(|| self.cancel_document(command))
            .or_else(|| self.cancel_attachment(command))
            .or_else(|| self.cancel_links(command))
            .or_else(|| self.cancel_refile(command))
        {
            return Some(action);
        }
        if let Some(action) = self.cancel_calendar_prompt(command) {
            return Some(action);
        }
        if let Some(action) = self.cancel_agenda_discovery(command) {
            return Some(action);
        }
        if let Some(action) = self.cancel_habit_prompt(command) {
            return Some(action);
        }
        if let Some(action) = self
            .timestamp_command(command)
            .or_else(|| self.agenda_presentation_command(command))
        {
            return Some(action);
        }
        if command == Command::ExecuteExtended {
            self.clock_recovery_pin = None;
            self.org.cancel_request();
            return Some(self.extended());
        }
        if command == Command::KeyboardQuit {
            self.clock_recovery_pin = None;
            self.org.cancel();
            self.agenda_query_plan = None;
            self.agenda_custom_plan = None;
            if matches!(
                self.pending,
                Some(
                    Pending::BookmarkReload(_)
                        | Pending::AgendaSources
                        | Pending::AgendaQuery(_)
                        | Pending::AgendaCustom(_)
                        | Pending::AgendaBulk(_)
                        | Pending::AgendaDate
                        | Pending::Habit(_)
                        | Pending::Timer(_)
                        | Pending::ClockReport(_)
                        | Pending::Clock(_)
                        | Pending::Timestamp(_)
                        | Pending::Property(_)
                        | Pending::Tags(_)
                        | Pending::Task(_)
                        | Pending::Estimate(_)
                )
            ) {
                self.pending = None;
            }
            if self.cancel_quit() {
                return Some(self.message("Quit"));
            }
            self.restore_search();
        }
        self.tree_input(command)
    }

    pub(super) fn command_outcome(&mut self, command: Command) -> Action {
        // C-g consumes pending focus in navigation without resetting the view.
        // Accepted names retire that stage before the shared handler takes over.
        if command != Command::KeyboardQuit {
            self.retire_command();
        }
        self.execute(command)
    }

    pub(super) fn retire_command(&mut self) {
        if self.input.focus() == Focus::Editing && matches!(self.pending, Some(Pending::Command)) {
            self.pending = None;
        }
    }

    pub(super) fn complete_argument(&mut self) -> Action {
        if matches!(self.pending, Some(Pending::Link(_) | Pending::Refile)) {
            return self.complete_source_target();
        }
        if matches!(self.pending, Some(Pending::Capture(_))) {
            return self.complete_capture();
        }
        if matches!(
            self.pending,
            Some(
                Pending::AgendaSources
                    | Pending::AgendaQuery(_)
                    | Pending::AgendaCustom(_)
                    | Pending::AgendaBulk(_)
                    | Pending::AgendaDate
                    | Pending::Habit(_)
                    | Pending::Timer(_)
                    | Pending::ClockReport(_)
                    | Pending::Clock(_)
                    | Pending::Timestamp(_)
                    | Pending::Task(_)
                    | Pending::Tags(_)
                    | Pending::Property(_)
                    | Pending::Estimate(_)
            )
        ) {
            return self.complete_org_argument();
        }
        match self.pending {
            Some(Pending::Path(_)) => self.complete_path(),
            Some(Pending::GotoLine { .. }) => {
                self.message("Enter a signed decimal line number; C-g cancels")
            }
            Some(Pending::Buffer { .. }) => self.complete_buffer(),
            Some(Pending::TreeCreate(_) | Pending::TreeRename(_)) => {
                self.message("Enter one name; C-g cancels")
            }
            Some(Pending::Write { .. }) => self.message("Enter a path; C-g cancels"),
            _ => self.message("No argument is active"),
        }
    }
    fn complete_org_argument(&mut self) -> Action {
        if self
            .pending
            .as_ref()
            .is_some_and(Pending::observation_argument)
        {
            return self.complete_observation_argument();
        }
        match self.pending {
            Some(Pending::Timestamp(_)) => self.complete_timestamp(),
            Some(Pending::Estimate(_)) => self.complete_estimate(),
            Some(Pending::Timer(_)) => self.message("Enter timer duration or yes/no; C-g cancels"),
            Some(Pending::Clock(_)) => {
                self.message("Choose the explicit clock action; C-g cancels")
            }
            Some(Pending::Task(_)) => {
                self.message("Enter a task state/selection key or note; C-g cancels")
            }
            Some(Pending::Tags(_)) => self.complete_tag(),
            Some(Pending::Property(_)) => self.message("Enter the property argument; C-g cancels"),
            _ => unreachable!("qualified Org completion"),
        }
    }
}

impl Session {
    fn complete_observation_argument(&mut self) -> Action {
        match self.pending {
            Some(Pending::AgendaSources) => {
                self.message("Enter current, buffers or one literal local path; C-g cancels")
            }
            Some(Pending::AgendaQuery(_) | Pending::AgendaCustom(_) | Pending::AgendaBulk(_)) => {
                self.message("Enter a typed query; C-g keeps the source or agenda")
            }
            Some(Pending::AgendaDate) => {
                self.message("Enter YYYY-MM-DD or today; C-g keeps the calendar")
            }
            Some(Pending::Habit(_)) => {
                self.message("Enter habit date or graph days/column; C-g cancels")
            }
            Some(Pending::ClockReport(_)) => {
                self.message("Choose the explicit clock action; C-g cancels")
            }
            _ => unreachable!("qualified observation argument"),
        }
    }
}

impl Session {
    fn complete_source_target(&mut self) -> Action {
        if matches!(self.pending, Some(Pending::Refile)) {
            self.complete_refile()
        } else {
            self.complete_link()
        }
    }
}
