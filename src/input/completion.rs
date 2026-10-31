use super::keymap::{Context, Sequence};
use super::{Input, Key, Outcome};
use crate::command::{Binding, Command, CompletionCommand as C};
use crossterm::event::KeyCode;

impl Input {
    pub(crate) fn completion_active(&self) -> bool {
        self.workspace_completion
    }
    pub(crate) fn set_completion(&mut self, active: bool) {
        if self.workspace_completion != active {
            self.workspace_completion = active;
            self.clear_prefix();
        }
    }
    // Escape remains a literal cancel here; outside the popup it retains Meta
    // timing. C-g keeps its globally reserved keyboard-quit configuration value.
    pub(super) fn completion_key(&mut self, key: Key) -> Option<Outcome> {
        if !self.workspace_completion {
            return None;
        }
        if key.plain() == Some(KeyCode::Esc) {
            return Some(self.completion_binding(C::Cancel));
        }
        let continued = !self.chords.is_empty();
        self.chords.push(key);
        let Some(sequence) = Sequence::from_chords(&self.chords) else {
            self.set_completion(false);
            return None;
        };
        let map = self.keymaps.map(Context::Completion);
        if let Some(Binding::Command(command)) = map.lookup(&sequence) {
            let command = match command {
                Command::Completion(command) => command,
                Command::KeyboardQuit => C::Cancel,
                _ => unreachable!("admitted popup command"),
            };
            return Some(self.completion_binding(command));
        }
        if map.is_prefix(&sequence) {
            return Some(self.wait_sequence(&sequence));
        }
        if continued {
            return Some(self.unmatched(key, true));
        }
        self.set_completion(false);
        None
    }
    fn completion_binding(&mut self, command: C) -> Outcome {
        self.clear_prefix();
        self.suppress_repeat |= matches!(command, C::Accept | C::Cancel);
        Outcome::Command(Command::Completion(command))
    }
    pub(super) fn completion_repeat(&self, key: Key) -> Option<bool> {
        if !self.workspace_completion {
            return None;
        }
        let sequence = Sequence::from_chords(&[key])?;
        self.keymaps
            .map(Context::Completion)
            .lookup(&sequence)
            .map(|binding| {
                matches!(
                    binding,
                    Binding::Command(Command::Completion(C::Next | C::Previous))
                )
            })
    }
}
