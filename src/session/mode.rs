use crate::buffer::Mode;

use super::{Action, Session};

impl Session {
    pub(super) fn sync_input_context(&mut self) {
        self.prepare_bookmark_markers();
        self.input.set_tree_focus(self.tree.focused());
        if self
            .bookmark_list
            .as_ref()
            .is_some_and(|s| self.windows.get(s.view).is_err())
        {
            self.bookmark_list = None;
        }
        self.input.set_bookmark_list(self.bookmark_list_active());
        if self
            .calendar_agenda
            .as_ref()
            .is_some_and(|s| self.windows.get(s.view).is_err())
        {
            self.calendar_agenda = None;
        }
        if self
            .query_agenda
            .as_ref()
            .is_some_and(|s| self.windows.get(s.view).is_err())
        {
            self.query_agenda = None;
        }
        let role = if self.source_edit_selected() {
            Some(crate::input::OrgContext::SourceEdit)
        } else if self.capture_selected() {
            Some(crate::input::OrgContext::Capture)
        } else if self.agenda_active() || self.habit_agenda.is_some() {
            Some(crate::input::OrgContext::Agenda)
        } else {
            self.buffer()
                .is_ok_and(|b| b.is_org())
                .then_some(crate::input::OrgContext::Editing)
        };
        self.input.set_org_context(role);
    }

    pub(super) fn select_mode(&mut self, mode: Mode) -> Action {
        if self.input.focus() != crate::input::Focus::Editing {
            return self.message("a prompt owns input focus");
        }
        let result = self
            .buffers
            .get_mut(self.view().point.stamp.buffer)
            .map_err(|error| error.to_string())
            .and_then(|buffer| buffer.select_mode(mode).map_err(|error| error.to_string()));
        match result {
            Ok(()) => self.message(if mode == Mode::Org {
                "Org mode"
            } else {
                "Text mode"
            }),
            Err(error) => self.error(&error),
        }
    }
}
