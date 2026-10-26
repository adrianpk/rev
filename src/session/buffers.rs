mod completion;
mod inventory;

use crate::buffer::{BufferId, Kill};
use crate::command::BufferCommand;
use crate::view::View;

use super::{Action, Pending, Session};

impl Session {
    pub(super) fn buffer_command(&mut self, command: BufferCommand) -> Action {
        if command == BufferCommand::List {
            return self.list_buffers();
        }
        let label = match command {
            BufferCommand::Switch => "Switch to buffer: ",
            _ => "Kill buffer: ",
        };
        if let Err(error) = self.input.argument(label, "") {
            return self.message(&error.to_string());
        }
        self.pending = Some(Pending::Buffer {
            command,
            default: self.view.point.stamp.buffer,
        });
        self.message("Tab completes names; Enter uses the current buffer when empty")
    }

    pub(super) fn buffer_argument(
        &mut self,
        command: BufferCommand,
        default: BufferId,
        name: &str,
    ) -> Action {
        let id = if name.is_empty() {
            Some(default)
        } else {
            self.buffers.named(name)
        };
        let Some(id) = id else {
            return self.message("No buffer with that name");
        };
        match command {
            BufferCommand::Switch => self.activate(id),
            BufferCommand::Kill => self.kill_buffer(id),
            _ => self.message("No buffer argument is pending"),
        }
    }

    pub(super) fn activate(&mut self, id: BufferId) -> Action {
        let buffer = match self.buffers.get(id) {
            Ok(buffer) => buffer,
            Err(error) => return self.message(&error.to_string()),
        };
        if self.view.point.stamp.buffer != id {
            self.view = View::new(buffer);
        }
        self.input.cancel();
        self.pending = None;
        self.message("")
    }

    fn kill_buffer(&mut self, id: BufferId) -> Action {
        let kill = match self.buffers.prepare_kill(id) {
            Ok(kill) => kill,
            Err(error) => return self.message(&error.to_string()),
        };
        let buffer = match self.buffers.get(id) {
            Ok(buffer) => buffer,
            Err(error) => return self.message(&error.to_string()),
        };
        if !buffer.is_modified() {
            return self.close_buffer(kill);
        }
        let message = format!("Close {:?}? y=save, n=discard, C-g=cancel", buffer.name());
        if let Err(error) = self.input.decision("Save before closing?") {
            return self.message(&error.to_string());
        }
        self.echo = message.clone();
        self.pending = Some(Pending::Kill { kill, message });
        Action::Redraw
    }

    pub(super) fn save_killing(&mut self, kill: Kill) -> Action {
        let buffer = match self.buffers.get(kill.id()) {
            Ok(buffer) => buffer,
            Err(error) => return self.message(&error.to_string()),
        };
        if buffer.file().is_none() {
            return self.write_argument(kill.id(), Some(kill));
        }
        self.save_to(kill.id(), None, Some(kill))
    }

    pub(super) fn close_buffer(&mut self, kill: Kill) -> Action {
        let selected = self.view.point.stamp.buffer == kill.id();
        let replacement = match self.buffers.kill(kill) {
            Ok(id) => id,
            Err(error) => return self.message(&error.to_string()),
        };
        if selected {
            self.activate(replacement);
        }
        self.input.cancel();
        self.pending = None;
        self.message("Buffer closed")
    }
}
