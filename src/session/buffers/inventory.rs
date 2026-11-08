use crate::buffer::{Buffer, BufferId};
use crate::input::Selection;

use super::{Action, Pending, Session};

impl Session {
    pub(in crate::session) fn list_buffers(&mut self) -> Action {
        if let Err(error) = self.input.inventory() {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::Inventory(
            self.terminal_buffer()
                .map(|b| b.id())
                .filter(|_| self.terminals.focused)
                .unwrap_or(self.windows.selected().point.stamp.buffer),
        ));
        self.message("Buffers: arrows move, Enter selects, C-g cancels")
    }

    pub(in crate::session) fn select_buffer(&mut self, selection: Selection) -> Action {
        let (id, terminals) = match self.pending {
            Some(Pending::Inventory(id)) => (id, false),
            Some(Pending::Terminals(id)) => (id, true),
            _ => return self.message("No buffer list is pending"),
        };
        if selection == Selection::Accept {
            if self.macros.active() && !terminals {
                let name = match self.buffers.get(id) {
                    Ok(buffer) => buffer.name().to_owned(),
                    Err(error) => return self.error(&error.to_string()),
                };
                return self.buffer_argument(crate::command::BufferCommand::Switch, id, &name);
            }
            return self.activate(id);
        }
        let Some(next) = self.inventory_target(id, selection, terminals) else {
            return self.message("Buffer list selection is no longer live");
        };
        self.pending = Some(if terminals {
            Pending::Terminals(next)
        } else {
            Pending::Inventory(next)
        });
        Action::Redraw
    }

    fn inventory_target(
        &self,
        id: BufferId,
        selection: Selection,
        terminals: bool,
    ) -> Option<BufferId> {
        let entries = || {
            self.buffers
                .iter()
                .filter(|b| !terminals || b.terminal().is_some())
        };
        let current = entries().position(|buffer| buffer.id() == id)?;
        let last = entries().count().checked_sub(1)?;
        let index = match selection {
            Selection::Next => current.saturating_add(1).min(last),
            Selection::Previous => current.saturating_sub(1),
            Selection::First => 0,
            Selection::Last => last,
            Selection::Accept => current,
        };
        entries().nth(index).map(Buffer::id)
    }
}
