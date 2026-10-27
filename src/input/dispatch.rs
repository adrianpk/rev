use crate::command::Binding;

use super::keymap::Sequence;
use super::{Input, Key, Outcome, Prefix};

impl Input {
    pub(super) fn mapped(&mut self, key: Key) -> Outcome {
        let continued = !self.chords.is_empty();
        self.chords.push(key);
        let Some(sequence) = Sequence::from_chords(&self.chords) else {
            return self.unmatched(key, continued);
        };
        let map = self.keymaps.map(self.map_context());
        if let Some(binding) = map.lookup(&sequence) {
            let custom = map.is_custom(&sequence);
            return self.bound(binding, key, custom, continued);
        }
        if map.is_prefix(&sequence) {
            return self.wait_sequence(&sequence);
        }
        self.unmatched(key, continued)
    }

    fn bound(&mut self, binding: Binding, key: Key, custom: bool, continued: bool) -> Outcome {
        self.clear_prefix();
        self.suppress_repeat |= continued || !self.binding_repeats(binding);
        self.context_binding(binding, key, custom)
    }

    pub(super) fn unmatched(&mut self, key: Key, continued: bool) -> Outcome {
        self.clear_prefix();
        if continued {
            self.suppress_repeat = true;
            return Outcome::Message("Undefined key sequence");
        }
        let outcome = self.focused(key);
        self.consume(outcome)
    }

    pub(super) fn wait_sequence(&mut self, sequence: &Sequence) -> Outcome {
        self.suppress_repeat = true;
        self.wait(sequence_prefix(sequence))
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
            self.suppress_repeat = true;
            self.workspace_tree = focused;
        }
    }

    pub(super) fn repeatable(&self, key: Key) -> bool {
        let Some(sequence) = Sequence::from_chords(&[key]) else {
            return false;
        };
        let map = self.keymaps.map(self.map_context());
        if let Some(binding) = map.lookup(&sequence) {
            return self.binding_repeats(binding);
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
