mod history;

use std::collections::{BTreeMap, VecDeque};

use crossterm::event::KeyCode;

use crate::command::Command;

use super::{Context, Error, Input, Key, Outcome, Prompt};

/// Owns bounded command text, candidate selection and history-navigation draft.
/// Candidates are admitted command identities, never interpreted as code.
#[derive(Debug)]
pub struct Palette {
    prompt: Prompt,
    query: Option<String>,
    selected: usize,
    // Explicit navigation overrides an exact draft. The default highlight
    // remains ready for Return, including when the query is empty.
    chosen: bool,
    top: usize,
    history: Option<usize>,
    draft: String,
    policy: crate::config::Minibuffer,
    ai_commands: Vec<Command>,
    usage: crate::command::usage::Usage,
    hints: BTreeMap<String, String>,
}

impl Palette {
    fn new(
        policy: &crate::config::Minibuffer,
        ai_commands: &[Command],
        usage: &crate::command::usage::Usage,
        hints: BTreeMap<String, String>,
    ) -> Result<Self, Error> {
        Ok(Self {
            prompt: Prompt::new("M-x ", "", policy.input_limit)?,
            query: None,
            selected: 0,
            chosen: false,
            top: 0,
            history: None,
            draft: String::new(),
            policy: policy.clone(),
            ai_commands: ai_commands.to_vec(),
            usage: usage.clone(),
            hints,
        })
    }

    pub fn prompt(&self) -> &Prompt {
        &self.prompt
    }

    /// Prepared annotation keyed by canonical identity, independent of rows,
    /// matching, ranking and prompt text.
    pub fn hint(&self, command: Command) -> Option<&str> {
        self.hints.get(command.name()).map(String::as_str)
    }

    pub(super) fn retire_hints(&mut self) {
        self.hints.clear();
    }

    pub fn candidates(&self) -> Vec<Command> {
        let query = self.query.as_deref().unwrap_or(self.prompt.text());
        let mut candidates: Vec<_> = Command::ALL
            .into_iter()
            .chain(self.ai_commands.iter().copied())
            .filter(|command| command.name().contains(query))
            .collect();
        if self.policy.r#match == crate::config::Match::Prefix {
            candidates.retain(|command| command.name().starts_with(query));
        }
        let rank = |command: &Command| {
            if command.name() == query {
                0
            } else if self.policy.r#match == crate::config::Match::PrefixThenSubstring
                && !command.name().starts_with(query)
            {
                2
            } else {
                1
            }
        };
        candidates.sort_by_key(rank);
        for group in candidates.chunk_by_mut(|a, b| rank(a) == rank(b)) {
            self.usage.rank(group);
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
        let previous = self.prompt.text().to_owned();
        let outcome = self.prompt.handle(key);
        self.edited(outcome, &previous)
    }

    pub(super) fn paste(&mut self, text: &str) -> Outcome {
        let previous = self.prompt.text().to_owned();
        let outcome = self.prompt.paste(text);
        self.edited(outcome, &previous)
    }

    fn edited(&mut self, outcome: Outcome, previous: &str) -> Outcome {
        if outcome == Outcome::Changed && self.prompt.text() != previous {
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
        let command = self
            .candidates()
            .iter()
            .find(|command| command.name() == self.prompt.text())
            .copied()
            .or_else(|| self.candidates().first().copied());
        match command {
            Some(command) => Outcome::Command(command),
            None if self.prompt.text().is_empty() => Outcome::Message("No command entered"),
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
    pub(crate) fn admit_ai_command(&mut self, name: crate::command::AiName) {
        let command = Command::AiDefined(name);
        if !self.ai_commands.contains(&command) {
            self.ai_commands.push(command);
        }
    }
    /// Opens command entry only from editing; session history survives cancel.
    pub fn commands(&mut self) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Command(Palette::new(
            &self.policy,
            &self.ai_commands,
            &self.command_usage.snapshot(),
            self.command_hints(),
        )?);
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
            // M-x, modal colon and configured accept aliases share this path.
            // Count before a handler can fail or open another argument prompt.
            self.command_usage.record(command);
            self.history.retain(|previous| *previous != command);
            self.history.push_front(command);
            self.history.truncate(self.policy.history_limit);
        }
        self.cancel();
        outcome
    }
}
