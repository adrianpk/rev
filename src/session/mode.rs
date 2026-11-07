use crate::buffer::Mode;

use super::{Action, Session};

impl Session {
    pub(super) fn sync_input_context(&mut self) {
        self.input.set_startup(self.startup_screen.is_some());
        if self.startup_screen.is_some() {
            return;
        }
        self.input.set_git(self.git_visible());
        self.input.set_git_commit(self.git_composing());
        if self.git_visible() {
            self.input.set_terminal_owner(None, false);
        }
        if self.git_composing() {
            self.input.set_tree_focus(false);
            self.input.set_bookmark_list(false);
            self.input.set_org_context(None);
            self.input
                .editing_owner(self.windows.selected_id(), self.view().point.stamp.buffer);
            return;
        }
        if self.git_visible() {
            return;
        }
        self.sync_terminal_focus();
        self.input.set_terminal_owner(
            self.terminals
                .focused
                .then(|| self.current_terminal())
                .flatten(),
            self.terminal_copy_active(),
        );
        self.sync_terminal_copies();
        self.qualify_repeat();
        if !self.sync_terminal_copy_input() {
            self.input
                .editing_owner(self.windows.selected_id(), self.view().point.stamp.buffer);
            self.sync_visual_context();
            self.input
                .observe_navigation(self.windows.selected_id(), self.view().point);
        }
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
        self.input.set_org_context(self.input_org_role());
    }

    fn input_org_role(&self) -> Option<crate::input::OrgContext> {
        if self.terminals.focused {
            return None;
        }
        if self.source_edit_selected() {
            Some(crate::input::OrgContext::SourceEdit)
        } else if self.capture_selected() {
            Some(crate::input::OrgContext::Capture)
        } else if self.agenda_active() || self.habit_agenda.is_some() {
            Some(crate::input::OrgContext::Agenda)
        } else {
            self.buffer()
                .is_ok_and(|b| b.is_org())
                .then_some(crate::input::OrgContext::Editing)
        }
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
