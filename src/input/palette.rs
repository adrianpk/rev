mod history;

use std::collections::VecDeque;

use crossterm::event::KeyCode;

use crate::command::Command;

use super::{Context, Error, Input, Key, Outcome, Prompt};

const HISTORY_LIMIT: usize = 128;

/// Owns bounded command text, completion cycling and history-navigation draft.
/// Candidates are derived from the static registry, never interpreted as code.
#[derive(Debug)]
pub struct Palette {
    prompt: Prompt,
    query: Option<String>,
    selected: usize,
    history: Option<usize>,
    draft: String,
}

impl Palette {
    fn new() -> Result<Self, Error> {
        Ok(Self {
            prompt: Prompt::new("M-x ", "")?,
            query: None,
            selected: 0,
            history: None,
            draft: String::new(),
        })
    }

    pub fn prompt(&self) -> &Prompt {
        &self.prompt
    }

    pub fn candidates(&self) -> Vec<Command> {
        Command::matching(self.query.as_deref().unwrap_or(self.prompt.text()))
    }

    /// A small inline candidate list; frame clipping owns available geometry.
    pub fn hint(&self) -> String {
        let names: Vec<_> = self
            .candidates()
            .into_iter()
            .take(3)
            .map(Command::name)
            .collect();
        if names.is_empty() {
            return "No matching commands".into();
        }
        names.join("  ")
    }

    fn handle(&mut self, key: Key, history: &VecDeque<Command>) -> Outcome {
        match key.plain() {
            Some(KeyCode::Tab) => self.complete(),
            Some(KeyCode::Enter) => self.accept(),
            _ => self.edit(key, history),
        }
    }

    fn edit(&mut self, key: Key, history: &VecDeque<Command>) -> Outcome {
        if let Some(previous) = history::direction(key) {
            return self.recall(history, previous);
        }
        let outcome = self.prompt.handle(key);
        self.edited(outcome)
    }

    pub(super) fn paste(&mut self, text: &str) -> Outcome {
        let outcome = self.prompt.paste(text);
        self.edited(outcome)
    }

    fn edited(&mut self, outcome: Outcome) -> Outcome {
        if outcome == Outcome::Changed {
            self.query = None;
            self.selected = 0;
            self.history = None;
        }
        outcome
    }

    fn complete(&mut self) -> Outcome {
        let candidates = self.candidates();
        if candidates.is_empty() {
            return Outcome::Message("No matching command");
        }
        if self.query.is_some() {
            self.selected = (self.selected + 1) % candidates.len();
        } else {
            self.query = Some(self.prompt.text().to_owned());
            self.selected = 0;
        }
        self.set_text(candidates[self.selected].name())
    }

    fn accept(&self) -> Outcome {
        if self.prompt.text().is_empty() {
            return Outcome::Message("No command entered");
        }
        let command =
            Command::named(self.prompt.text()).or_else(|| self.candidates().first().copied());
        match command {
            Some(command) => Outcome::Command(command),
            None => Outcome::Message("Unknown command"),
        }
    }

    fn set_text(&mut self, text: &str) -> Outcome {
        match Prompt::new("M-x ", text) {
            Ok(prompt) => {
                self.prompt = prompt;
                Outcome::Changed
            }
            Err(_) => Outcome::Message("Invalid command history entry"),
        }
    }
}

impl Input {
    /// Opens command entry only from editing; session history survives cancel.
    pub fn commands(&mut self) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Command(Palette::new()?);
        self.prefix = None;
        self.meta_deadline = None;
        Ok(())
    }

    pub fn palette(&self) -> Option<&Palette> {
        match &self.context {
            Context::Command(palette) => Some(palette),
            _ => None,
        }
    }

    pub(super) fn command_key(&mut self, key: Key) -> Outcome {
        let Context::Command(palette) = &mut self.context else {
            return Outcome::Ignored;
        };
        let outcome = palette.handle(key, &self.history);
        if key.plain() != Some(KeyCode::Enter) {
            return outcome;
        }
        if let Outcome::Command(command) = outcome {
            self.history.retain(|previous| *previous != command);
            self.history.push_front(command);
            self.history.truncate(HISTORY_LIMIT);
        }
        self.cancel();
        outcome
    }
}
