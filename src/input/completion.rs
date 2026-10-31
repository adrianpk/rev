use crossterm::event::KeyCode;

use crate::command::{Command, CompletionCommand as C};

use super::{Input, Key, Outcome};

impl Input {
    pub(crate) fn set_completion(&mut self, active: bool) {
        if self.workspace_completion != active {
            self.workspace_completion = active;
            self.clear_prefix();
        }
    }

    // This overlay preserves source focus. Unrelated keys dismiss it before
    // normal keymap dispatch, including prefixes, paste and pane navigation.
    pub(super) fn completion_key(&mut self, key: Key) -> Option<Outcome> {
        if !self.workspace_completion {
            return None;
        }
        let Some(command) = command(key) else {
            self.set_completion(false);
            return None;
        };
        self.clear_prefix();
        self.suppress_repeat |= matches!(command, C::Accept | C::Cancel);
        Some(Outcome::Command(Command::Completion(command)))
    }
}

fn command(key: Key) -> Option<C> {
    if key.control('g') {
        return Some(C::Cancel);
    }
    if key.control('n') {
        return Some(C::Next);
    }
    if key.control('p') {
        return Some(C::Previous);
    }
    match key.plain() {
        Some(KeyCode::Down) => Some(C::Next),
        Some(KeyCode::Up) => Some(C::Previous),
        Some(KeyCode::Tab | KeyCode::Enter) => Some(C::Accept),
        Some(KeyCode::Esc) => Some(C::Cancel),
        _ => None,
    }
}
