use crossterm::event::{KeyCode, KeyModifiers};

use crate::buffer::edit::Point;
use crate::command::navigation::Target;
use crate::command::{Binding, Command, Navigation};
use crate::window::ViewId;

use super::keymap::Sequence;
use super::{EditingState, Input, Key, Outcome, Prefix};

const STROKES: u8 = 16;

/// Pending syntax is bounded and qualified by the exact native source point.
#[derive(Debug, Default)]
pub(super) struct Grammar {
    pub count: Option<u32>,
    pub strokes: u8,
    pub character: Option<Navigation>,
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

    fn navigation_pending(&self) -> bool {
        self.grammar.strokes > 0 || self.grammar.character.is_some()
    }

    pub(crate) fn expect_navigation(&mut self, request: Navigation) -> Outcome {
        self.grammar.character = Some(request);
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
        if !self.modal_editing() || self.editing_state != EditingState::Normal {
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

    fn target_character(&mut self, mut request: Navigation, key: Key) -> Outcome {
        let strokes = self.grammar.strokes;
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
        request.character = Some(character);
        self.suppress_repeat = true;
        Outcome::Command(Command::Navigate(request))
    }

    pub(super) fn navigation_binding(&mut self, binding: Binding) -> Result<Binding, Outcome> {
        let count = self.grammar.count;
        let strokes = self.grammar.strokes;
        self.clear_prefix();
        let Binding::Command(Command::Navigate(mut request)) = binding else {
            return if count.is_some() {
                Err(Outcome::Message(
                    "This command does not accept a modal count",
                ))
            } else {
                Ok(binding)
            };
        };
        request.count = count;
        if matches!(request.target, Target::Find(_)) && request.character.is_none() {
            self.grammar.strokes = strokes;
            return Err(self.expect_navigation(request));
        }
        Ok(Binding::Command(Command::Navigate(request)))
    }

    pub(super) fn navigation_label(&self) -> Option<String> {
        if let Some(request) = self.grammar.character {
            return Some(format!("{}: character", request.name()));
        }
        self.grammar.count.map(|count| {
            let suffix = Sequence::from_chords(&self.chords)
                .map(|s| format!(" {s}"))
                .unwrap_or_default();
            format!("{count}{suffix}")
        })
    }
}
