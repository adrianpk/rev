use crossterm::event::KeyCode;

use super::{Input, Key, Outcome};
use crate::command::{Command, TreeCommand};

impl Input {
    pub(crate) fn suppress_command_repeat(&mut self) {
        self.suppress_repeat = true;
    }

    pub(super) fn tree_key(&mut self, key: Key) -> Outcome {
        self.suppress_repeat = true;
        let command = match key.plain() {
            Some(KeyCode::Char('t')) => TreeCommand::Toggle,
            Some(KeyCode::Char('f')) => TreeCommand::Focus,
            Some(KeyCode::Char('r')) => TreeCommand::Refresh,
            Some(KeyCode::Char('R')) => TreeCommand::Rename,
            Some(KeyCode::Char('o')) => TreeCommand::Open,
            Some(KeyCode::Char('n')) => TreeCommand::CreateFile,
            Some(KeyCode::Char('d')) => TreeCommand::CreateDirectory,
            _ => return Outcome::Message("Undefined key sequence"),
        };
        Outcome::Command(Command::Tree(command))
    }
}
