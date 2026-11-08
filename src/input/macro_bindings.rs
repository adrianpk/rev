use super::keymap::{Context as MapContext, Sequence};
use super::{Context, Error, Focus, Input, Key, Outcome, Prompt};
use crate::command::{Binding, Command, MacroName};
use crossterm::event::KeyCode;
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct Capture {
    keys: Vec<Key>,
    context: MapContext,
    local: Option<MapContext>,
    modal: bool,
    pub prompt: Prompt,
    binding: bool,
}

pub(crate) fn overlaps(a: &Sequence, b: &Sequence) -> bool {
    a.chords().starts_with(b.chords()) || b.chords().starts_with(a.chords())
}

impl Input {
    pub(crate) fn macro_catalog(
        &mut self,
        names: Vec<(MacroName, Option<String>)>,
        bindings: BTreeMap<Sequence, Command>,
    ) {
        self.macro_commands = names
            .iter()
            .map(|(name, _)| Command::MacroNamed(*name))
            .collect();
        self.macro_descriptions = names
            .into_iter()
            .filter_map(|(name, description)| {
                description.map(|text| {
                    (
                        name.name().into(),
                        text.chars()
                            .map(|c| if c.is_control() { ' ' } else { c })
                            .collect(),
                    )
                })
            })
            .collect();
        self.macro_bindings = bindings;
        self.retire_command_hints();
        self.clear_prefix();
    }

    pub(crate) fn command_named(&self, name: &str) -> Option<Command> {
        Command::named(name).or_else(|| {
            self.ai_commands
                .iter()
                .chain(&self.macro_commands)
                .copied()
                .find(|command| command.name() == name)
        })
    }

    pub(super) fn macro_bindings_active(&self) -> bool {
        self.focus() == Focus::Editing
            && !self.workspace_startup
            && !self.workspace_tree
            && self.workspace_macro.is_none()
            && !self.workspace_bookmarks
            && !self.workspace_git
            && self.workspace_terminal.is_none()
            && !matches!(
                self.workspace_org,
                Some(
                    super::OrgContext::Agenda
                        | super::OrgContext::Calendar
                        | super::OrgContext::Capture
                )
            )
    }

    pub(super) fn macro_bound(&mut self, key: Key) -> Option<Outcome> {
        if !self.macro_bindings_active() {
            return None;
        }
        let mut keys = self.chords.clone();
        keys.push(key);
        let sequence = Sequence::from_chords(&keys)?;
        if let Some(command) = self.macro_bindings.get(&sequence).copied() {
            let continued = !self.chords.is_empty();
            self.suppress_repeat = true;
            return Some(self.bound(Binding::Command(command), key, true, continued));
        }
        if self
            .macro_bindings
            .keys()
            .any(|target| target.chords().starts_with(sequence.chords()))
        {
            self.chords = keys;
            return Some(self.wait_sequence(&sequence));
        }
        None
    }

    pub(crate) fn macro_key_capture(&mut self) -> Result<(), Error> {
        self.macro_key_capture_for(true)
    }
    pub(crate) fn macro_literal_key_capture(&mut self) -> Result<(), Error> {
        self.macro_key_capture_for(false)
    }
    fn macro_key_capture_for(&mut self, binding: bool) -> Result<(), Error> {
        self.available()?;
        self.context = Context::MacroKey(Capture {
            keys: Vec::new(),
            binding,
            context: self.map_context(),
            local: self.workspace_org.map(super::OrgContext::keymap),
            modal: self.modal_editing(),
            prompt: Prompt::new(
                if binding {
                    "Bind macro key: "
                } else {
                    "Insert macro key: "
                },
                "",
                self.policy.input_limit,
            )?,
        });
        self.clear_prefix();
        Ok(())
    }

    pub(super) fn macro_key(&mut self, key: Key) -> Outcome {
        let Context::MacroKey(capture) = &mut self.context else {
            return Outcome::Ignored;
        };
        if capture.keys.len() == Sequence::MAX_CHORDS {
            return Outcome::Message("Macro binding exceeds eight admitted chords");
        }
        capture.keys.push(key);
        let Some(mut sequence) = Sequence::from_chords(&capture.keys) else {
            capture.keys.pop();
            return Outcome::Message("Key has no admitted native binding notation");
        };
        if capture.binding
            && capture.keys.len() == 1
            && let Some(KeyCode::Char(c)) = key.plain()
            && (c.is_ascii_digit() || c.is_ascii_uppercase())
        {
            sequence = Sequence::parse(&format!("C-x C-k {c}")).expect("reserved macro slot");
        } else {
            let map = if capture.modal {
                self.keymaps.vim_map(capture.context, capture.local)
            } else {
                self.keymaps.map(capture.context)
            };
            let prefix = map.is_prefix(&sequence)
                || self.macro_bindings.keys().any(|target| {
                    target != &sequence && target.chords().starts_with(sequence.chords())
                });
            let argument = matches!(map.lookup(&sequence), Some(Binding::Command(command)) if command.prefix_control());
            if prefix && !argument && capture.keys.len() < Sequence::MAX_CHORDS {
                if capture.prompt.replace(&sequence.to_string()).is_err() {
                    return Outcome::Message("Macro key display exceeds the native prompt limit");
                }
                return Outcome::Changed;
            }
        }
        self.context = Context::Editing;
        self.clear_prefix();
        self.suppress_repeat = true;
        Outcome::MacroKey(sequence)
    }

    /// Inspect every ordinary profile map before replacing a source binding.
    /// The layer never participates in prompts, components or raw terminals.
    pub(crate) fn macro_binding_check(&self, sequence: &Sequence) -> Result<bool, String> {
        self.macro_binding_check_except(sequence, None)
    }

    pub(crate) fn macro_library_binding_check(
        &self,
        sequence: &Sequence,
        name: MacroName,
    ) -> Result<bool, String> {
        self.macro_binding_check_except(sequence, Some(name))
    }

    fn macro_binding_check_except(
        &self,
        sequence: &Sequence,
        except: Option<MacroName>,
    ) -> Result<bool, String> {
        let first = sequence.chords()[0];
        if first.modifiers == crossterm::event::KeyModifiers::ALT
            && matches!(first.code, KeyCode::Char(c) if c.is_ascii_digit() || c == '-')
        {
            return Err("Macro bindings cannot shadow native prefix arguments".into());
        }
        if sequence
            .chords()
            .iter()
            .any(|key| key.control('g') || key.control(']'))
        {
            return Err("Macro bindings cannot replace reserved cancellation".into());
        }
        let mut occupied = self.macro_bindings.iter().any(|(key, target)| {
            !matches!(target, Command::MacroNamed(n) if Some(*n) == except)
                && overlaps(key, sequence)
        });
        for context in MapContext::ALL {
            for (key, target) in self.keymaps.map(context).bindings() {
                if !overlaps(key, sequence) {
                    continue;
                }
                if matches!(target, Binding::Command(Command::KeyboardQuit)) {
                    return Err("Macro bindings cannot shadow native cancellation".into());
                }
                if matches!(target, Binding::Command(command) if command.prefix_control())
                    && key.chords().len() <= sequence.chords().len()
                {
                    return Err("Macro bindings cannot shadow native prefix arguments".into());
                }
                occupied = true;
            }
        }
        Ok(occupied)
    }
}
