use crossterm::event::KeyCode;

use super::{Context, Error, Input, Key, Outcome, Prompt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchAction {
    Next,
    Accept,
}

impl Input {
    pub fn search(&mut self) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Search(Prompt::new("I-search: ", "")?);
        self.prefix = None;
        self.meta_deadline = None;
        Ok(())
    }
}

pub(super) fn handle(prompt: &mut Prompt, key: Key) -> Outcome {
    if key.control('s') {
        return Outcome::Search(SearchAction::Next);
    }
    match key.plain() {
        Some(KeyCode::Enter) => Outcome::Search(SearchAction::Accept),
        Some(KeyCode::Tab) => prompt.literal_paste("\t"),
        _ => prompt.handle(key),
    }
}
