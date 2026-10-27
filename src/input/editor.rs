use crate::command::{Binding, Command};

use super::keymap::{Context, Sequence};
use super::{Input, Key, Outcome, Prefix};

impl Input {
    pub(super) fn editing(&mut self, key: Key) -> Outcome {
        if self.workspace_tree {
            return self.legacy_editing(key);
        }
        let continued = !self.chords.is_empty();
        self.chords.push(key);
        let Some(sequence) = Sequence::from_chords(&self.chords) else {
            return self.unmatched(key, continued);
        };
        let map = self.keymaps.map(Context::Editor);
        if let Some(Binding::Command(command)) = map.lookup(&sequence) {
            return self.editor_command(command, continued);
        }
        if map.is_prefix(&sequence) {
            self.suppress_repeat = true;
            return self.wait(sequence_prefix(&sequence));
        }
        self.unmatched(key, continued)
    }

    fn editor_command(&mut self, command: Command, continued: bool) -> Outcome {
        self.clear_prefix();
        self.suppress_repeat |= continued || !command.repeatable();
        Outcome::Command(command)
    }

    fn unmatched(&mut self, key: Key, continued: bool) -> Outcome {
        self.clear_prefix();
        if continued {
            self.suppress_repeat = true;
            return Outcome::Message("Undefined key sequence");
        }
        self.literal(key)
    }

    /// The bounded pending sequence is input state, independent of buffer text.
    /// Built-in prefixes keep their public identities and existing notices.
    pub fn prefix_label(&self) -> Option<String> {
        let prefix = self.prefix()?;
        if prefix == Prefix::Sequence {
            return Sequence::from_chords(&self.chords).map(|sequence| sequence.to_string());
        }
        Some(prefix.label().into())
    }

    pub(super) fn clear_prefix(&mut self) {
        self.prefix = None;
        self.chords.clear();
        self.meta_deadline = None;
    }

    pub(crate) fn set_tree_focus(&mut self, focused: bool) {
        if self.workspace_tree != focused {
            self.clear_prefix();
            self.workspace_tree = focused;
        }
    }

    pub(super) fn repeatable(&self, key: Key) -> bool {
        if self.focus() != super::Focus::Editing || self.workspace_tree {
            return key.repeatable();
        }
        let Some(sequence) = Sequence::from_chords(&[key]) else {
            return false;
        };
        let map = self.keymaps.map(Context::Editor);
        if let Some(Binding::Command(command)) = map.lookup(&sequence) {
            return command.repeatable();
        }
        !map.is_prefix(&sequence) && matches!(key.plain(), Some(crossterm::event::KeyCode::Char(_)))
    }
}

fn sequence_prefix(sequence: &Sequence) -> Prefix {
    match sequence.to_string().as_str() {
        "C-x" => Prefix::ControlX,
        "C-x r" => Prefix::Rectangle,
        "C-x t" => Prefix::Tree,
        _ => Prefix::Sequence,
    }
}
