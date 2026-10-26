use crossterm::event::KeyCode;

use super::{Input, Key, Outcome};
use crate::command::{Command, TreeCommand};

impl Input {
    pub(super) fn tree_key(&mut self, key: Key) -> Outcome {
        self.suppress_repeat = true;
        let command = match key.plain() {
            Some(KeyCode::Char('t')) => TreeCommand::Toggle,
            Some(KeyCode::Char('f')) => TreeCommand::Focus,
            _ => return Outcome::Message("Undefined key sequence"),
        };
        Outcome::Command(Command::Tree(command))
    }
}
