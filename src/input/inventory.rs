use crossterm::event::KeyCode;

use super::{Context, Error, Input, Key, Outcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Next,
    Previous,
    First,
    Last,
    Accept,
}

impl Input {
    pub fn inventory(&mut self) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Inventory;
        self.prefix = None;
        self.meta_deadline = None;
        Ok(())
    }

    /// Completion replaces only the active bounded argument, keeping focus.
    pub fn complete(&mut self, text: &str) -> Result<(), Error> {
        let Context::Argument(prompt) = &mut self.context else {
            return Err(Error::Busy);
        };
        prompt.replace(text)?;
        Ok(())
    }
}

pub(super) fn handle(key: Key) -> Outcome {
    let selection = match key.plain() {
        Some(KeyCode::Down) => Selection::Next,
        Some(KeyCode::Up) => Selection::Previous,
        Some(KeyCode::Home) => Selection::First,
        Some(KeyCode::End) => Selection::Last,
        Some(KeyCode::Enter) => Selection::Accept,
        _ => return Outcome::Message("Use arrows, Enter or C-g"),
    };
    Outcome::Selection(selection)
}
