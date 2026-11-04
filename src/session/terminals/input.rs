use alacritty_terminal::term::TermMode;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventState, KeyModifiers, MouseEvent, MouseEventKind,
};

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
        if let Some(view) = self.terminal_pointer_view(mouse) {
            return Some(view);
        }
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

    fn terminal_pointer_view(&mut self, mouse: MouseEvent) -> Option<Action> {
        let composition = self.composition()?;
        let (view, _) = composition.panels().iter().find(|(id, panel)| {
            let area = panel.outer();
            let contains = (area.column()..area.column() + area.width()).contains(&mouse.column)
                && (area.row()..area.row() + area.height()).contains(&mouse.row);
            contains
                && self
                    .windows
                    .get(*id)
                    .ok()
                    .and_then(|v| self.buffers.get(v.point.stamp.buffer).ok())
                    .is_some_and(|b| b.terminal().is_some())
        })?;
        if matches!(mouse.kind, MouseEventKind::Down(_)) {
            return Some(match self.select_view(*view) {
                Ok(()) => Action::Redraw,
                Err(error) => self.error(&error.to_string()),
            });
        }
        Some(Action::Wait)
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
