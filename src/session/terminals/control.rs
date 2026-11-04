use alacritty_terminal::term::TermMode;
use crossterm::event::{KeyCode, KeyModifiers};

use super::{Action, Session};
use crate::{command::TerminalCommand, input::Key};

impl Session {
    pub(super) fn terminal_control_command(&mut self, command: TerminalCommand) -> Option<Action> {
        let key = match command {
            TerminalCommand::Send(key) => Key {
                code: key.code(),
                modifiers: KeyModifiers::NONE,
            },
            TerminalCommand::Clear => Key {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::CONTROL,
            },
            TerminalCommand::Undo => Key {
                code: KeyCode::Char('_'),
                modifiers: KeyModifiers::CONTROL,
            },
            _ => return None,
        };
        if self.terminal_copy_active() {
            return Some(self.error("Exit terminal copy mode before sending input"));
        }
        let mode = self
            .current_terminal()
            .and_then(|id| self.terminals.sessions.get(id))
            .map_or(TermMode::empty(), |session| *session.screen().mode());
        let bytes =
            super::keyboard::encode(key, mode, false).expect("admitted explicit terminal key");
        Some(self.terminal_send(&bytes))
    }
    pub(super) fn terminal_clear_scrollback(&mut self) -> Action {
        let result = self.terminal_input_id().and_then(|id| {
            self.terminals
                .sessions
                .clear_scrollback(id)
                .map_err(|error| error.to_string())?;
            self.terminals.copies.retain(|_, copy| copy.id != id);
            if self.terminals.selected == Some(id) {
                self.terminals.viewport = Default::default();
            }
            let views: Vec<_> = self
                .windows
                .iter()
                .filter(|(_, view)| {
                    self.buffers
                        .get(view.point.stamp.buffer)
                        .is_ok_and(|buffer| buffer.terminal() == Some(id))
                })
                .map(|(id, _)| id)
                .collect();
            for view in views {
                self.windows.get_mut(view).expect("observed view").terminal = Default::default();
            }
            Ok(())
        });
        match result {
            Ok(()) => {
                self.input.cancel();
                self.message("Terminal scrollback cleared; following the tail")
            }
            Err(error) => self.error(&error),
        }
    }
}
