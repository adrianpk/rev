use crossterm::event::KeyCode;

use super::{Context, Error, Input, Key, Outcome, Prompt, Selection};

impl Input {
    pub(crate) fn path_argument(&mut self) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Path(Prompt::new("Find file: ", "", self.policy.input_limit)?);
        self.clear_prefix();
        Ok(())
    }
}

pub(super) fn handle(prompt: &mut Prompt, key: Key) -> Outcome {
    match key.plain() {
        Some(KeyCode::Up) => Outcome::Selection(Selection::Previous),
        Some(KeyCode::Down) => Outcome::Selection(Selection::Next),
        _ if key.control('p') => Outcome::Selection(Selection::Previous),
        _ if key.control('n') => Outcome::Selection(Selection::Next),
        _ => prompt.handle(key),
    }
}
