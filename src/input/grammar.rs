use crossterm::event::{KeyCode, KeyModifiers};

use crate::buffer::edit::Point;
use crate::command::editing::Operand;
use crate::command::{Binding, Command, Editing, Navigation};
mod operator;
#[derive(Debug, Clone, Copy)]
pub(super) enum Character {
    Navigation(Navigation),
    Editing(Editing),
}
use crate::window::ViewId;

use super::keymap::Sequence;
use super::{Input, Key, Outcome, Prefix};

const STROKES: u8 = 16;

/// Pending syntax is bounded and qualified by the exact native source point.
#[derive(Debug, Default)]
pub(super) struct Grammar {
    pub count: Option<u32>,
    pub strokes: u8,
    pub character: Option<Character>,
    pub operator: Option<Editing>,
    pub object: Option<bool>,
}

impl Input {
    pub(crate) fn observe_navigation(&mut self, view: ViewId, point: Point) {
        let observation = Some((view, point));
        if self.navigation_source != observation {
            if self.navigation_pending() {
                self.clear_prefix();
            }
            self.navigation_source = observation;
        }
    }

    pub(crate) fn editing_composition(&self) -> bool {
        self.modal_editing() && !self.editing_state.text_entry() && self.navigation_pending()
    }

    pub(crate) fn navigation_pending(&self) -> bool {
        self.grammar.strokes > 0
            || self.grammar.character.is_some()
            || self.grammar.operator.is_some()
    }

    pub(crate) fn expect_navigation(&mut self, request: Navigation) -> Outcome {
        self.grammar.character = Some(Character::Navigation(request));
        self.suppress_repeat = true;
        self.wait(Prefix::Sequence)
    }

    pub(super) fn grammar_key(&mut self, key: Key) -> Option<Outcome> {
        if self.navigation_pending()
            && key.code == KeyCode::Char('x')
            && key.modifiers == KeyModifiers::ALT
        {
            self.clear_prefix();
            return None;
        }
        if let Some(request) = self.grammar.character {
            return Some(self.target_character(request, key));
        }
        if let Some(outcome) = self.object_key(key) {
            return Some(outcome);
        }
        if !self.modal_editing() || self.editing_state.text_entry() {
            return None;
        }
        self.grammar.strokes += 1;
        if self.grammar.strokes > STROKES {
            self.clear_prefix();
            return Some(Outcome::Message("Modal input exceeds 16 strokes"));
        }
        self.count_digit(key)
    }

    fn count_digit(&mut self, key: Key) -> Option<Outcome> {
        if !self.chords.is_empty() || self.grammar.count.is_none() && self.custom_digit(key) {
            return None;
        }
        let Some(KeyCode::Char(character)) = key.plain() else {
            return None;
        };
        let digit = character
            .to_digit(10)
            .filter(|_| character.is_ascii_digit())?;
        if digit == 0 && self.grammar.count.is_none() {
            return None;
        }
        let count = self
            .grammar
            .count
            .unwrap_or(0)
            .checked_mul(10)
            .and_then(|n| n.checked_add(digit));
        let Some(count) = count else {
            self.clear_prefix();
            return Some(Outcome::Message("Modal count exceeds u32"));
        };
        self.grammar.count = Some(count);
        Some(self.wait(Prefix::Sequence))
    }

    fn custom_digit(&self, key: Key) -> bool {
        Sequence::from_chords(&[key]).is_some_and(|sequence| self.active_map().is_custom(&sequence))
    }

    fn target_character(&mut self, request: Character, key: Key) -> Outcome {
        let strokes = self.grammar.strokes;
        let operator = self.grammar.operator;
        self.clear_prefix();
        if strokes >= STROKES {
            return Outcome::Message("Modal input exceeds 16 strokes");
        }
        let Some(KeyCode::Char(character)) = key.plain() else {
            return Outcome::Message("Find requires one Unicode character");
        };
        if character.is_control() {
            return Outcome::Message("Find requires a printable character");
        }
        self.suppress_repeat = true;
        match request {
            Character::Navigation(mut request) => {
                request.character = Some(character);
                if let Some(mut operator) = operator {
                    operator.operand = Some(Operand::Motion(request));
                    Outcome::Command(Command::Edit(operator))
                } else {
                    Outcome::Command(Command::Navigate(request))
                }
            }
            Character::Editing(mut request) => {
                request.character = Some(character);
                Outcome::Command(Command::Edit(request))
            }
        }
    }

    pub(super) fn navigation_binding(&mut self, binding: Binding) -> Result<Binding, Outcome> {
        let count = self.grammar.count;
        match binding {
            Binding::Command(Command::Edit(mut request)) => {
                request.count = count;
                self.editing_binding(request)
            }
            Binding::Command(Command::Navigate(mut request)) => {
                request.count = count;
                self.motion_binding(request)
            }
            _ => {
                let pending = self.grammar.operator.is_some() || count.is_some();
                self.clear_prefix();
                if pending {
                    Err(Outcome::Message("Unsupported operator/count target"))
                } else {
                    Ok(binding)
                }
            }
        }
    }

    pub(super) fn navigation_label(&self) -> Option<String> {
        if let Some(request) = self.grammar.character {
            let name = match request {
                Character::Navigation(r) => r.name(),
                Character::Editing(r) => r.name(),
            };
            return Some(format!("{name}: character"));
        }
        if let Some(request) = self.grammar.operator {
            let count = self
                .grammar
                .count
                .map(|n| n.to_string())
                .unwrap_or_default();
            return Some(format!("{} {count}", request.name()));
        }
        self.grammar.count.map(|count| {
            let suffix = Sequence::from_chords(&self.chords)
                .map(|s| format!(" {s}"))
                .unwrap_or_default();
            format!("{count}{suffix}")
        })
    }
}
