mod completion;
mod inventory;

use crate::buffer::{BufferId, Kill};
use crate::command::BufferCommand;

use super::{Action, AfterSave, Pending, Session};

impl Session {
    pub(super) fn buffer_command(&mut self, command: BufferCommand) -> Action {
        if command == BufferCommand::List {
            return self.list_buffers();
        }
        let label = match command {
            BufferCommand::Switch => "Switch to buffer: ",
            _ => "Kill buffer: ",
        };
        let argument = if command == BufferCommand::Switch {
            self.input.names_argument(label, "", self.buffer_names())
        } else {
            self.input.argument(label, "")
        };
        if let Err(error) = argument {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::Buffer {
            command,
            default: self
                .terminal_buffer()
                .map(|b| b.id())
                .filter(|_| self.terminals.focused)
                .unwrap_or(self.windows.selected().point.stamp.buffer),
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
            if self.macros.active() {
                return self.stop_macro(
                    crate::macros::StopReason::NativeFailure,
                    "Macro buffer name is unavailable",
                );
            }
            return self.message("No buffer with that name");
        };
        if self.macros.active()
            && self
                .buffers
                .get(id)
                .is_ok_and(|buffer| buffer.terminal().is_some())
        {
            return self.stop_macro(
                crate::macros::StopReason::Unsupported,
                "Macro cannot transfer to a terminal",
            );
        }
        let accepted = self
            .buffers
            .get(id)
            .expect("resolved buffer")
            .name()
            .to_owned();
        if !self.macro_accept_argument(crate::macros::Role::Buffer, &accepted) {
            return Action::Redraw;
        }
        match command {
            BufferCommand::Switch => self.activate(id),
            BufferCommand::Kill => self.kill_buffer(id),
            _ => self.message("No buffer argument is pending"),
        }
    }

    pub(super) fn activate(&mut self, id: BufferId) -> Action {
        let buffer = match self.buffers.get(id) {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        if let Some(terminal) = buffer.terminal() {
            self.input.cancel();
            self.pending = None;
            return self.select_terminal(terminal);
        }
        if self.windows.selected().point.stamp.buffer != id {
            let view = self.context_view(buffer);
            let jump = match self.prepare_jump_to(buffer, view.point) {
                Ok(jump) => jump,
                Err(error) => return self.error(&error.to_string()),
            };
            *self.windows.selected_mut() = view;
            self.publish_jump(jump);
        }
        self.touch_completion_source(id);
        self.terminal_source_focus();
        self.input.cancel();
        self.pending = None;
        self.follow_tree();
        self.message("")
    }

    pub(super) fn kill_buffer(&mut self, id: BufferId) -> Action {
        if self.active_clock.as_ref().is_some_and(|c| c.buffer() == id) {
            return self.clock_departure(super::outline::clocks::Departure::Close(id));
        }
        let kill = match self.buffers.prepare_kill(id) {
            Ok(kill) => kill,
            Err(error) => return self.error(&error.to_string()),
        };
        let buffer = match self.buffers.get(id) {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        if !buffer.is_modified() {
            if let Some(terminal) = buffer.terminal()
                && self
                    .terminals()
                    .get(terminal)
                    .is_some_and(|s| s.owns_process())
                && !self.terminal_closing(terminal)
            {
                let message = format!(
                    "Terminate {:?}? y=terminate, n=keep, C-g=cancel",
                    buffer.name()
                );
                if let Err(error) = self.input.decision("Terminate terminal?") {
                    return self.error(&error.to_string());
                }
                self.echo = message.clone();
                self.pending = Some(Pending::TerminalClose { kill, message });
                return Action::Redraw;
            }
            return self.close_buffer(kill);
        }
        let message = format!("Close {:?}? y=save, n=discard, C-g=cancel", buffer.name());
        if let Err(error) = self.input.decision("Save before closing?") {
            return self.error(&error.to_string());
        }
        self.echo_kind = crate::display::Message::Normal;
        self.echo = message.clone();
        self.pending = Some(Pending::Kill { kill, message });
        Action::Redraw
    }

    pub(super) fn save_killing(&mut self, kill: Kill) -> Action {
        let buffer = match self.buffers.get(kill.id()) {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        if buffer.file().is_none() {
            return self.write_argument(kill.id(), AfterSave::Close(kill));
        }
        self.save_to(kill.id(), None, AfterSave::Close(kill))
    }

    pub(super) fn close_buffer(&mut self, kill: Kill) -> Action {
        let closed = kill.id();
        if let Some(terminal) = self.buffers.get(closed).ok().and_then(|b| b.terminal())
            && self.terminals().get(terminal).is_some()
        {
            return self.close_terminal(terminal, kill);
        }
        self.input.cancel();
        self.pending = None;
        self.remove_buffer(kill)
    }

    pub(super) fn remove_buffer(&mut self, kill: Kill) -> Action {
        let closed = kill.id();
        let replacement = match self.buffers.kill(kill) {
            Ok(id) => id,
            Err(error) => return self.error(&error.to_string()),
        };
        let buffer = match self.buffers.get(replacement) {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        self.context_positions.remove(&closed);
        self.windows.replace_buffer(closed, buffer);
        self.lsp.closed(closed);
        self.recent_clocks.retain(|p| p.stamp.buffer != closed);
        self.refresh_buffer_completion(closed);
        self.focus_message("Buffer closed")
    }
}
