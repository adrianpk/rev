use crate::search::Direction;
use crossterm::event::KeyCode;

use super::{Context, Error, Input, Key, Outcome, Prompt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchAction {
    Repeat(Direction),
    Accept,
}

impl Input {
    pub fn search(&mut self, direction: Direction) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Search(Prompt::new(label(direction), "", self.policy.input_limit)?);
        self.clear_prefix();
        Ok(())
    }

    pub(crate) fn search_label(&mut self, direction: Direction) {
        if let Context::Search(prompt) = &mut self.context {
            prompt.set_label(label(direction));
        }
    }
}

pub(super) fn handle(prompt: &mut Prompt, key: Key) -> Outcome {
    if key.control('s') {
        prompt.set_label(label(Direction::Forward));
        return Outcome::Search(SearchAction::Repeat(Direction::Forward));
    }
    if key.control('r') {
        prompt.set_label(label(Direction::Backward));
        return Outcome::Search(SearchAction::Repeat(Direction::Backward));
    }
    match key.plain() {
        Some(KeyCode::Enter) => Outcome::Search(SearchAction::Accept),
        Some(KeyCode::Tab) => prompt.literal_paste("\t"),
        _ => prompt.handle(key),
    }
}

fn label(direction: Direction) -> &'static str {
    match direction {
        Direction::Forward => "I-search: ",
        Direction::Backward => "I-search backward: ",
    }
}
