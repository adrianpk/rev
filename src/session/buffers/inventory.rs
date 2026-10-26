use crate::buffer::{Buffer, BufferId};
use crate::input::Selection;

use super::{Action, Pending, Session};

impl Session {
    pub(in crate::session) fn list_buffers(&mut self) -> Action {
        if let Err(error) = self.input.inventory() {
            return self.message(&error.to_string());
        }
        self.pending = Some(Pending::Inventory(
            self.windows.selected().point.stamp.buffer,
        ));
        self.message("Buffers: arrows move, Enter selects, C-g cancels")
    }

    pub(in crate::session) fn select_buffer(&mut self, selection: Selection) -> Action {
        let Some(Pending::Inventory(id)) = self.pending else {
            return self.message("No buffer list is pending");
        };
        if selection == Selection::Accept {
            return self.activate(id);
        }
        let Some(next) = self.inventory_target(id, selection) else {
            return self.message("Buffer list selection is no longer live");
        };
        self.pending = Some(Pending::Inventory(next));
        Action::Redraw
    }

    fn inventory_target(&self, id: BufferId, selection: Selection) -> Option<BufferId> {
        let current = self.buffers.iter().position(|buffer| buffer.id() == id)?;
        let last = self.buffers.iter().len().checked_sub(1)?;
        let index = match selection {
            Selection::Next => current.saturating_add(1).min(last),
            Selection::Previous => current.saturating_sub(1),
            Selection::First => 0,
            Selection::Last => last,
            Selection::Accept => current,
        };
        self.buffers.iter().nth(index).map(Buffer::id)
    }
}
