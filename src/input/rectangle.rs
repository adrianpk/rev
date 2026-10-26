use crossterm::event::{KeyCode, KeyModifiers};

use crate::command::{Command, RectangleCommand};

use super::{Input, Key, Outcome};

impl Input {
    pub(super) fn rectangle_key(&mut self, key: Key) -> Outcome {
        self.suppress_repeat = true;
        if key.code == KeyCode::Char('w') && key.modifiers == KeyModifiers::ALT {
            return Outcome::Command(Command::Rectangle(RectangleCommand::Copy));
        }
        if key.plain() == Some(KeyCode::Char('k')) {
            return Outcome::Command(Command::Rectangle(RectangleCommand::Kill));
        }
        if key.plain() == Some(KeyCode::Char('y')) {
            return Outcome::Command(Command::Rectangle(RectangleCommand::Yank));
        }
        Outcome::Message("Undefined key sequence")
    }
}
