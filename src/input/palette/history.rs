use std::collections::VecDeque;

use crossterm::event::KeyCode;

use crate::command::Command;
use crate::input::{Key, Outcome};

use super::Palette;

pub(super) fn direction(key: Key) -> Option<bool> {
    match key.plain() {
        Some(KeyCode::Up) => Some(true),
        Some(KeyCode::Down) => Some(false),
        _ => control_direction(key),
    }
}

fn control_direction(key: Key) -> Option<bool> {
    if key.control('p') {
        return Some(true);
    }
    key.control('n').then_some(false)
}

impl Palette {
    pub(super) fn recall(&mut self, history: &VecDeque<Command>, previous: bool) -> Outcome {
        if history.is_empty() {
            return Outcome::Message("Command history is empty");
        }
        let index = match (self.history, previous) {
            (None, true) => Some(0),
            (Some(index), true) => Some((index + 1).min(history.len() - 1)),
            (Some(0), false) => None,
            (Some(index), false) => Some(index - 1),
            _ => return Outcome::Changed,
        };
        let text = index.map_or_else(
            || self.draft.clone(),
            |index| history[index].name().to_owned(),
        );
        let draft = self.prompt.text().to_owned();
        let outcome = self.set_text(&text);
        if outcome == Outcome::Changed {
            if self.history.is_none() && previous {
                self.draft = draft;
            }
            self.history = index;
            self.query = None;
            self.selected = 0;
        }
        outcome
    }
}
