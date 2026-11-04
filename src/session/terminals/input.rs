use alacritty_terminal::term::TermMode;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventState, KeyModifiers};

use super::super::{Action, Session};
use crate::input::{Focus, Key};

impl Session {
    /// Host transients and already-started host prefixes retain first refusal.
    /// Other live keys bypass source/modal editing in either startup profile.
    pub(in crate::session) fn terminal_input(&mut self, event: &Event) -> Option<Action> {
        if self.terminal_host_transient() {
            return None;
        }
        if let Event::Mouse(mouse) = event {
            return self.terminal_pointer(*mouse);
        }
        if !self.terminals.focused {
            return None;
        }
        match event {
            Event::Key(event) => self.terminal_key(*event),
            Event::Paste(text) => Some(self.terminal_send(text.as_bytes())),
            _ => Some(Action::Wait),
        }
    }

    pub(in crate::session) fn terminal_host_transient(&self) -> bool {
        self.input.focus() != Focus::Editing
            || self.pending.is_some()
            || self.input.prefix().is_some()
    }

    fn terminal_key(&mut self, event: KeyEvent) -> Option<Action> {
        let Some(key) = Key::normalize(event) else {
            return Some(Action::Wait);
        };
        let id = self.current_terminal();
        let literal = self
            .terminals
            .literal
            .take()
            .is_some_and(|armed| Some(armed) == id);
        if !literal {
            if key.code == KeyCode::Char('q') && key.modifiers == KeyModifiers::CONTROL {
                return Some(self.terminal_literal_next());
            }
            if key.code == KeyCode::Char('x') && key.modifiers == KeyModifiers::ALT {
                return Some(self.execute(crate::command::Command::ExecuteExtended));
            }
            if self.input.terminal_host_key(key) {
                return None;
            }
        }
        let mode = id
            .and_then(|id| self.terminals.sessions.get(id))
            .map_or(TermMode::empty(), |s| *s.screen().mode());
        let Some(bytes) =
            super::keyboard::encode(key, mode, event.state.contains(KeyEventState::KEYPAD))
        else {
            return Some(self.message("Key has no terminal encoding"));
        };
        Some(self.terminal_send(&bytes))
    }

    pub(super) fn terminal_literal_next(&mut self) -> Action {
        let Some(id) = self.current_terminal().filter(|_| self.terminals.focused) else {
            return self.error("Literal terminal input requires a focused terminal");
        };
        self.terminals.literal = Some(id);
        self.message("Terminal: send the next key literally")
    }

    pub(in crate::session) fn terminal_send(&mut self, bytes: &[u8]) -> Action {
        let usable = if self.terminals.view.is_some() {
            self.buffer_geometry(&self.windows, self.terminals.visible)
                .is_some()
        } else {
            self.terminal_rows().is_some()
        };
        if !usable {
            return self.error("Terminal presentation is suspended at this screen size");
        }
        let Some(id) = self.current_terminal() else {
            return self.error("No terminal is selected");
        };
        match self.terminals.sessions.queue(id, bytes) {
            Ok(()) => Action::Redraw,
            Err(error) => self.error(&format!("Terminal input: {error}")),
        }
    }
}
