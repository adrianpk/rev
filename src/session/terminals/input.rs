use alacritty_terminal::term::TermMode;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventState, KeyModifiers};

use super::super::{Action, Session};
use crate::input::{Focus, Key};

impl Session {
    /// Host transients and already-started host prefixes retain first refusal.
    /// Other live keys bypass source/modal editing in either startup profile.
    pub(in crate::session) fn terminal_input(&mut self, event: &Event) -> Option<Action> {
        if self.git_visible() {
            return None;
        }
        if self.terminal_host_transient() {
            return None;
        }
        if let Event::Mouse(mouse) = event {
            if self.terminals.copy_chord.is_some() {
                return Some(Action::Wait);
            }
            return self.terminal_pointer(*mouse);
        }
        if !self.terminals.focused {
            return None;
        }
        if self.terminal_copy_active() {
            return self.terminal_copy_input(event);
        }

        match event {
            Event::Key(event) => self.terminal_key(*event),
            Event::Paste(text) => Some(self.terminal_paste_text(text)),
            _ => Some(Action::Wait),
        }
    }

    fn terminal_copy_input(&mut self, event: &Event) -> Option<Action> {
        match event {
            Event::Key(key)
                if Key::normalize(*key)
                    .is_some_and(|key| key.code == KeyCode::Enter && key.modifiers.is_empty()) =>
            {
                Some(self.terminal_copy_done())
            }
            Event::Paste(_) => Some(self.error("Terminal copy text is read-only")),
            _ => None,
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
        if let Some(id) = self.terminals.copy_chord.take()
            && self.current_terminal() == Some(id)
        {
            return Some(
                self.terminal_copy_chord(key, event.state.contains(KeyEventState::KEYPAD)),
            );
        }
        let id = self.current_terminal();
        let literal = self
            .terminals
            .literal
            .take()
            .is_some_and(|armed| Some(armed) == id);
        if !literal {
            if let Some(action) = self.terminal_host_input(key) {
                return Some(action);
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

    fn terminal_host_input(&mut self, key: Key) -> Option<Action> {
        if crate::input::Input::terminal_copy_prefix(key) {
            self.terminals.copy_chord = self.current_terminal();
            return Some(self.message("Terminal C-c: C-t copy, C-c send interrupt, C-g cancel"));
        }
        use crate::command::{Command, TerminalCommand};
        Some(match crate::input::Input::terminal_host_command(key)? {
            Command::ExecuteExtended => self.execute(Command::ExecuteExtended),
            Command::Terminal(TerminalCommand::SendNextKey) => self.terminal_literal_next(),
            Command::Terminal(TerminalCommand::Yank) => self.terminal_yank(false),
            Command::Terminal(TerminalCommand::YankPop) => self.terminal_yank(true),
            _ => unreachable!("terminal host input command"),
        })
    }

    fn terminal_copy_chord(&mut self, key: Key, keypad: bool) -> Action {
        if key.modifiers == KeyModifiers::CONTROL {
            match key.code {
                KeyCode::Char('t') => return self.terminal_copy_mode(),
                KeyCode::Char('g') => return self.message("Terminal prefix cancelled"),
                KeyCode::Char('c') => return self.terminal_send(b"\x03"),
                _ => {}
            }
        }
        let mode = self
            .current_terminal()
            .and_then(|id| self.terminals.sessions.get(id))
            .map_or(TermMode::empty(), |s| *s.screen().mode());
        let Some(mut bytes) = super::keyboard::encode(key, mode, keypad) else {
            return self.error("Key has no terminal encoding");
        };
        bytes.insert(0, 3);
        self.terminal_send(&bytes)
    }

    pub(super) fn terminal_literal_next(&mut self) -> Action {
        if self.terminal_copy_active() {
            return self.error("Exit terminal copy mode before sending input");
        }
        let Some(id) = self.current_terminal().filter(|_| self.terminals.focused) else {
            return self.error("Literal terminal input requires a focused terminal");
        };
        self.terminals.literal = Some(id);
        self.message("Terminal: send the next key literally")
    }

    pub(in crate::session) fn terminal_send(&mut self, bytes: &[u8]) -> Action {
        let result = self.terminal_input_id().and_then(|id| {
            self.terminals
                .sessions
                .queue(id, bytes)
                .map_err(|error| error.to_string())
        });
        match result {
            Ok(()) => {
                self.terminals.yank = None;
                self.resume_terminal_input_tail();
                Action::Redraw
            }
            Err(error) => self.error(&format!("Terminal input: {error}")),
        }
    }
    pub(super) fn terminal_input_id(&self) -> Result<crate::term::SessionId, String> {
        if !self.terminals.focused {
            return Err("Terminal input requires a focused terminal".into());
        }
        let usable = if self.terminals.view.is_some() {
            self.buffer_geometry(&self.windows, self.terminals.visible)
                .is_some()
        } else {
            self.terminal_rows().is_some()
        };
        if !usable {
            return Err("Terminal presentation is suspended at this screen size".into());
        }
        self.current_terminal()
            .ok_or("No terminal is selected".into())
    }
}
