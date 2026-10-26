use crossterm::event::KeyCode;

use crate::command::Command;

use super::{Context, Error, Input, Key, Outcome, Prompt};

pub(crate) const QUESTION: &str = "Replace occurrence? [y/n/q]";

impl Input {
    pub(crate) fn replace_argument(&mut self, label: &'static str) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Literal(Prompt::new(label, "")?);
        self.prefix = None;
        self.meta_deadline = None;
        Ok(())
    }

    pub(crate) fn replace_decision(&mut self) -> Result<(), Error> {
        self.available()?;
        self.context = Context::ReplaceDecision;
        self.prefix = None;
        self.meta_deadline = None;
        Ok(())
    }
}

pub(super) fn argument(prompt: &mut Prompt, key: Key) -> Outcome {
    match key.plain() {
        Some(KeyCode::Tab) => prompt.literal_paste("\t"),
        _ => prompt.handle(key),
    }
}

pub(super) fn answer(key: Key) -> Outcome {
    match key.plain() {
        Some(KeyCode::Char('y' | 'Y')) => Outcome::Decision(true),
        Some(KeyCode::Char('n' | 'N')) => Outcome::Decision(false),
        Some(KeyCode::Char('q' | 'Q')) => Outcome::Command(Command::KeyboardQuit),
        _ => Outcome::Message("Answer y, n or q"),
    }
}
