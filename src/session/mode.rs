use crate::buffer::Mode;

use super::{Action, Session};

impl Session {
    pub(super) fn sync_input_context(&mut self) {
        self.input.set_tree_focus(self.tree.focused());
        let active = self.buffer().is_ok_and(|buffer| buffer.is_org());
        self.input
            .set_org_context(active.then_some(if self.habit_agenda.is_some() {
                crate::input::OrgContext::Agenda
            } else {
                crate::input::OrgContext::Editing
            }));
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
