use alacritty_terminal::term::TermMode;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use super::super::{Action, Session};
use crate::input::{Focus, Key};

impl Session {
    /// Host transients and already-started host prefixes retain first refusal.
    /// Other live keys bypass source/modal editing in either startup profile.
    pub(in crate::session) fn terminal_input(&mut self, event: &Event) -> Option<Action> {
        if self.input.focus() != Focus::Editing || self.pending.is_some() {
            return None;
        }
        if let Event::Mouse(mouse) = event {
            return self.terminal_pointer(*mouse);
        }
        if !self.terminals.focused || self.input.prefix().is_some() {
            return None;
        }
        match event {
            Event::Key(event) => self.terminal_key(*event),
            Event::Paste(text) => Some(self.terminal_send(text.as_bytes())),
            _ => Some(Action::Wait),
        }
    }

    fn terminal_pointer(&mut self, mouse: MouseEvent) -> Option<Action> {
        let panel = self.composition().and_then(|c| c.terminal());
        if panel.is_some_and(|p| {
            let a = p.outer();
            (a.column()..a.column() + a.width()).contains(&mouse.column)
                && (a.row()..a.row() + a.height()).contains(&mouse.row)
        }) {
            if matches!(mouse.kind, MouseEventKind::Down(_)) {
                if !self.terminals.focused {
                    self.terminals.home = Some((self.windows.selected_id(), self.tree.focused()));
                }
                self.terminals.focused = true;
                self.tree.edit();
                self.sync_input_context();
            }
            return Some(Action::Redraw);
        }
        if matches!(mouse.kind, MouseEventKind::Down(_)) {
            self.terminal_source_focus();
        }
        None
    }

    fn terminal_key(&mut self, event: KeyEvent) -> Option<Action> {
        let Some(key) = Key::normalize(event) else {
            return Some(Action::Wait);
        };
        if self.input.terminal_host_key(key) {
            return None;
        }
        let mode = self
            .current_terminal()
            .and_then(|id| self.terminals.sessions.get(id))
            .is_some_and(|s| s.screen().mode().contains(TermMode::APP_CURSOR));
        let Some(bytes) = encode(key, mode) else {
            return Some(self.message("Key has no terminal encoding"));
        };
        Some(self.terminal_send(&bytes))
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

fn encode(key: Key, application: bool) -> Option<Vec<u8>> {
    let mut bytes = if let KeyCode::Char(c) = key.code {
        character(c, key.modifiers)?
    } else {
        special(key.code, application)?.to_vec()
    };
    if key.modifiers.contains(KeyModifiers::ALT) {
        bytes.insert(0, 0x1b);
    }
    Some(bytes)
}

fn character(c: char, modifiers: KeyModifiers) -> Option<Vec<u8>> {
    if modifiers.contains(KeyModifiers::CONTROL) {
        return match c {
            ' '..='?' => Some(vec![(c as u8) & 0x1f]),
            '@'..='_' | 'a'..='z' => Some(vec![(c as u8) & 0x1f]),
            _ => None,
        };
    }
    Some(c.to_string().into_bytes())
}

fn special(code: KeyCode, application: bool) -> Option<&'static [u8]> {
    match code {
        KeyCode::Enter => Some(b"\r"),
        KeyCode::Tab => Some(b"\t"),
        KeyCode::Backspace => Some(b"\x7f"),
        KeyCode::Esc => Some(b"\x1b"),
        KeyCode::Delete => Some(b"\x1b[3~"),
        _ => cursor_key(code, application),
    }
}

fn cursor_key(code: KeyCode, application: bool) -> Option<&'static [u8]> {
    let keys: [(KeyCode, &[u8], &[u8]); 4] = [
        (KeyCode::Up, b"\x1b[A", b"\x1bOA"),
        (KeyCode::Down, b"\x1b[B", b"\x1bOB"),
        (KeyCode::Right, b"\x1b[C", b"\x1bOC"),
        (KeyCode::Left, b"\x1b[D", b"\x1bOD"),
    ];
    keys.into_iter()
        .find(|(key, _, _)| *key == code)
        .map(|(_, normal, app)| if application { app } else { normal })
}
