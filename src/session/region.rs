use crate::command::RegionCommand;
use crate::input::Focus;

use super::{Action, Session};

impl Session {
    pub(super) fn region_command(&mut self, command: RegionCommand) -> Action {
        if self.input.focus() != Focus::Editing {
            return self.message("a prompt owns input focus");
        }
        let result = self.change_region(command);
        match result {
            Ok(()) => self.message(if command == RegionCommand::Set {
                "Mark set"
            } else {
                ""
            }),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn change_region(&mut self, command: RegionCommand) -> Result<(), crate::buffer::edit::Error> {
        let point = self.windows.selected().point;
        let buffer = self.buffers.get_mut(point.stamp.buffer)?;
        match command {
            RegionCommand::Set => buffer.activate_mark(point)?,
            RegionCommand::Exchange => {
                let target = buffer.exchange_mark(point)?;
                let view = self.windows.selected_mut();
                view.point = target;
                view.reset_navigation();
            }
        }
        Ok(())
    }

    pub(super) fn keyboard_quit(&mut self) -> Action {
        self.tree.edit();
        for (_, view) in self.windows.iter() {
            view.cancel_layout();
        }
        let buffer = match self
            .buffers
            .get_mut(self.windows.selected().point.stamp.buffer)
        {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        buffer.deactivate_region();
        self.input.cancel();
        self.message("Quit")
    }
}
