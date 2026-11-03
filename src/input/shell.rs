use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyModifiers};

use super::{Context, Error, Input, Key, Outcome, Prompt};

/// Accepted shell text is session-local data, separate from named-command history.
#[derive(Debug, Default)]
pub(super) struct History {
    commands: VecDeque<String>,
    index: Option<usize>,
    draft: String,
}

impl Input {
    pub(crate) fn shell_prompt(&mut self) -> Result<(), Error> {
        self.available()?;
        let prompt = Prompt::new("Shell command: ", "", self.policy.input_limit.min(4096))?;
        self.shell_history.index = None;
        self.shell_history.draft.clear();
        self.context = Context::Shell(prompt);
        self.clear_prefix();
        Ok(())
    }

    /// Called only after the session admits the command and owns its operation.
    pub(crate) fn remember_shell(&mut self, command: &str) {
        self.shell_history.commands.push_front(command.to_owned());
        self.shell_history.commands.truncate(32);
    }
}

impl History {
    pub(super) fn handle(&mut self, prompt: &mut Prompt, key: Key) -> Outcome {
        let previous = match (key.modifiers, key.code) {
            (KeyModifiers::ALT, KeyCode::Char('p')) => true,
            (KeyModifiers::ALT, KeyCode::Char('n')) => false,
            _ => return prompt.handle(key),
        };
        if self.commands.is_empty() {
            return Outcome::Message("Shell command history is empty");
        }
        self.recall(prompt, previous)
    }

    fn recall(&mut self, prompt: &mut Prompt, previous: bool) -> Outcome {
        let index = match (self.index, previous) {
            (None, true) => Some(0),
            (Some(index), true) => Some((index + 1).min(self.commands.len() - 1)),
            (Some(0), false) => None,
            (Some(index), false) => Some(index - 1),
            _ => return Outcome::Changed,
        };
        let text = index.map_or(self.draft.as_str(), |i| self.commands[i].as_str());
        let draft = prompt.text().to_owned();
        if let Err(error) = prompt.replace(text) {
            return Outcome::Message(match error {
                Error::Limit => super::prompt::LIMIT_MESSAGE,
                _ => "Shell history entry is unavailable",
            });
        }
        if self.index.is_none() && previous {
            self.draft = draft;
        }
        self.index = index;
        Outcome::Changed
    }
}
