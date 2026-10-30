use crate::input::Focus;

use super::{Action, Session};

impl Session {
    pub(super) fn toggle_wrap(&mut self) -> Action {
        if self.input.focus() != Focus::Editing || self.tree.focused() {
            return self.message("Visual Line mode requires an editing view");
        }
        let id = self.view().point.stamp.buffer;
        let enabled = match self.buffers.get_mut(id) {
            Ok(buffer) => buffer.toggle_soft_wrap(),
            Err(error) => return self.error(&error.to_string()),
        };
        self.message(if enabled {
            "Visual Line mode enabled"
        } else {
            "Visual Line mode disabled"
        })
    }
}
