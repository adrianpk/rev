use crate::command::Binding;

use super::keymap::Sequence;
use super::{Input, Key, Outcome, Prefix};

impl Input {
    pub(super) fn mapped(&mut self, key: Key) -> Outcome {
        if let Some(outcome) = self.argument_entry(key) {
            return outcome;
        }
        if let Some(outcome) = self.grammar_key(key) {
            return outcome;
        }
        let continued = !self.chords.is_empty();
        self.chords.push(key);
        let Some(sequence) = Sequence::from_chords(&self.chords) else {
            return self.unmatched(key, continued);
        };
        let map = self.active_map();
        if let Some(binding) = map.lookup(&sequence) {
            let custom = map.is_custom(&sequence);
            return self.bound(binding, key, custom, continued);
        }
        if map.is_prefix(&sequence) {
            return self.wait_sequence(&sequence);
        }
        self.unmatched(key, continued)
    }

    pub(super) fn bound(
        &mut self,
        binding: Binding,
        key: Key,
        custom: bool,
        continued: bool,
    ) -> Outcome {
        if self.focus() == super::Focus::Editing
            && let Binding::Command(command) = binding
            && command.prefix_control()
        {
            return self.prefix_control(command, Some(key));
        }
        if self.prefix_argument().supplied()
            && (self.grammar.count.is_some() || self.grammar.operator.is_some())
        {
            self.clear_prefix();
            return Outcome::Message("Editor prefix conflicts with modal count/operator");
        }
        let binding = match self.navigation_binding(binding) {
            Ok(binding) => binding,
            Err(outcome) => return outcome,
        };
        self.suppress_repeat |= continued || !self.binding_repeats(binding);
        self.context_binding(binding, key, custom)
    }

    pub(super) fn unmatched(&mut self, key: Key, continued: bool) -> Outcome {
        let operator = self.grammar.operator.is_some();
        self.clear_sequence();
        if operator {
            return Outcome::Message("Undefined operator target");
        }
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
        if let Some(label) = self.argument_label() {
            return Some(label);
        }
        if let Some(label) = self.navigation_label() {
            return Some(label);
        }
        let prefix = self.prefix()?;
        if prefix == Prefix::Sequence {
            return Sequence::from_chords(&self.chords).map(|sequence| sequence.to_string());
        }
        Some(prefix.label().into())
    }

    pub(super) fn clear_prefix(&mut self) {
        self.clear_argument();
        self.clear_sequence();
    }

    pub(super) fn clear_sequence(&mut self) {
        self.grammar = Default::default();
        self.prefix = None;
        self.chords.clear();
        self.meta_deadline = None;
    }

    pub(crate) fn set_startup(&mut self, shown: bool) {
        if self.workspace_startup != shown {
            self.retire_command_hints();
            self.clear_prefix();
            self.suppress_repeat = true;
            self.workspace_startup = shown;
        }
    }
    pub(crate) fn set_tree_focus(&mut self, focused: bool) {
        if self.workspace_tree != focused {
            self.retire_command_hints();
            self.clear_prefix();
            self.suppress_repeat = true;
            self.workspace_tree = focused;
        }
    }

    pub(super) fn repeatable(&self, key: Key) -> bool {
        if let Some(repeatable) = self.completion_repeat(key) {
            return repeatable;
        }
        let Some(sequence) = Sequence::from_chords(&[key]) else {
            return false;
        };
        let map = self.active_map();
        if let Some(binding) = map.lookup(&sequence) {
            return self.binding_repeats(binding);
        }
        self.accepts_text()
            && !map.is_prefix(&sequence)
            && matches!(key.plain(), Some(crossterm::event::KeyCode::Char(_)))
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
