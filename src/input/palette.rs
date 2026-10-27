mod history;

use std::collections::VecDeque;

use crossterm::event::KeyCode;

use crate::command::Command;

use super::{Context, Error, Input, Key, Outcome, Prompt};

/// Owns bounded command text, candidate selection and history-navigation draft.
/// Candidates are derived from the static registry, never interpreted as code.
#[derive(Debug)]
pub struct Palette {
    prompt: Prompt,
    query: Option<String>,
    selected: usize,
    chosen: bool,
    top: usize,
    history: Option<usize>,
    draft: String,
    policy: crate::config::Minibuffer,
}

impl Palette {
    fn new(policy: &crate::config::Minibuffer) -> Result<Self, Error> {
        Ok(Self {
            prompt: Prompt::new("M-x ", "", policy.input_limit)?,
            query: None,
            selected: 0,
            chosen: false,
            top: 0,
            history: None,
            draft: String::new(),
            policy: policy.clone(),
        })
    }

    pub fn prompt(&self) -> &Prompt {
        &self.prompt
    }

    pub fn candidates(&self) -> Vec<Command> {
        let query = self.query.as_deref().unwrap_or(self.prompt.text());
        let mut candidates = Command::matching(query);
        match self.policy.r#match {
            crate::config::Match::PrefixThenSubstring => {}
            crate::config::Match::Prefix => {
                candidates.retain(|command| command.name().starts_with(query))
            }
            crate::config::Match::Substring => candidates.sort_by_key(|command| command.name()),
        }
        candidates
    }

    /// Candidate rows only; the query has its own reserved row. Even an empty
    /// result requests one row so the empty state has an explicit surface.
    pub fn requested_rows(&self) -> u16 {
        self.candidates()
            .len()
            .max(1)
            .min(self.policy.candidate_limit) as u16
    }

    pub fn selected(&self) -> Option<usize> {
        (self.selected < self.candidates().len()).then_some(self.selected)
    }

    pub fn top(&self) -> usize {
        self.top
    }

    /// Geometry changes presentation only. Zero candidate rows retain both
    /// selection and origin so a suspended list can be restored coherently.
    fn reveal(&mut self, rows: u16) {
        if rows == 0 {
            return;
        }
        let rows = usize::from(rows);
        if self.selected < self.top {
            self.top = self.selected;
        } else if self.selected >= self.top + rows {
            self.top = self.selected + 1 - rows;
        }
        self.top = self.top.min(self.candidates().len().saturating_sub(rows));
    }

    fn handle(&mut self, key: Key, history: &VecDeque<Command>) -> Outcome {
        if let Some(previous) = direction(key) {
            return self.navigate(previous);
        }
        match key.plain() {
            Some(KeyCode::Tab) => self.complete(),
            Some(KeyCode::Enter) => self.accept(),
            _ => self.edit(key, history),
        }
    }

    fn navigate(&mut self, previous: bool) -> Outcome {
        let count = self.candidates().len();
        if count == 0 {
            return Outcome::Message("No matching command");
        }
        self.selected = if previous {
            self.selected.saturating_sub(1)
        } else {
            (self.selected + 1).min(count - 1)
        };
        self.chosen = true;
        Outcome::Changed
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
            self.chosen = false;
            self.top = 0;
            self.history = None;
        }
        outcome
    }

    fn complete(&mut self) -> Outcome {
        let candidates = self.candidates();
        if candidates.is_empty() {
            return Outcome::Message("No matching command");
        }
        let query = self
            .query
            .clone()
            .unwrap_or_else(|| self.prompt.text().to_owned());
        let outcome = self.set_text(candidates[self.selected].name());
        if outcome == Outcome::Changed {
            self.query = Some(query);
        }
        outcome
    }

    fn accept(&self) -> Outcome {
        if self.chosen {
            return Outcome::Command(self.candidates()[self.selected]);
        }
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
        match self.prompt.replace(text) {
            Ok(()) => Outcome::Changed,
            Err(_) => Outcome::Message(super::prompt::LIMIT_MESSAGE),
        }
    }
}

fn direction(key: Key) -> Option<bool> {
    match key.plain() {
        Some(KeyCode::Up) => Some(true),
        Some(KeyCode::Down) => Some(false),
        _ if key.control('p') => Some(true),
        _ if key.control('n') => Some(false),
        _ => None,
    }
}

impl Input {
    /// Opens command entry only from editing; session history survives cancel.
    pub fn commands(&mut self) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Command(Palette::new(&self.policy)?);
        self.clear_prefix();
        Ok(())
    }

    pub fn palette(&self) -> Option<&Palette> {
        match &self.context {
            Context::Command(palette) => Some(palette),
            _ => None,
        }
    }

    pub(crate) fn reveal_commands(&mut self, rows: u16) {
        if let Context::Command(palette) = &mut self.context {
            palette.reveal(rows);
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
            self.history.truncate(self.policy.history_limit);
        }
        self.cancel();
        outcome
    }
}
