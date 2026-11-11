use crate::command::{Command, MacroCommand};
use crate::input::{Focus, Outcome};
use crate::macros::Repeat;
use crate::session::Session;

impl Session {
    /// Input has already normalized Escape and releases. Replace only a plain
    /// editing outcome; unrelated input keeps its original native dispatch.
    pub(in crate::session) fn macro_repeat_outcome(&mut self, outcome: Outcome) -> Outcome {
        if self.macros.expanded || self.macros.runner.is_some() {
            return outcome;
        }
        let Some(repeat) = self.macros.repeat else {
            return outcome;
        };
        if matches!(outcome, Outcome::Ignored) {
            return outcome;
        }
        let key = self.input.logical_key;
        let command = if self.input.focus() == Focus::Editing && self.pending.is_none() {
            key.and_then(|key| repeat_command(repeat, key))
        } else {
            None
        };
        if let Some(command) = command {
            self.input.cancel();
            Outcome::Command(Command::Macro(command))
        } else {
            self.macros.repeat = None;
            outcome
        }
    }
}

fn repeat_command(repeat: Repeat, key: crate::input::Key) -> Option<MacroCommand> {
    match repeat {
        Repeat::EndCall
            if key.code == crossterm::event::KeyCode::Char('e') && key.modifiers.is_empty() =>
        {
            Some(MacroCommand::EndAndCall)
        }
        Repeat::Ring if key.modifiers == crossterm::event::KeyModifiers::CONTROL => {
            ring_command(key.code)
        }
        _ => None,
    }
}

fn ring_command(code: crossterm::event::KeyCode) -> Option<MacroCommand> {
    match code {
        crossterm::event::KeyCode::Char('k') => Some(MacroCommand::CallRepeat),
        crossterm::event::KeyCode::Char('l') => Some(MacroCommand::CallSecond),
        crossterm::event::KeyCode::Char('n') => Some(MacroCommand::Next),
        crossterm::event::KeyCode::Char('p') => Some(MacroCommand::Previous),
        crossterm::event::KeyCode::Char('d') => Some(MacroCommand::Delete),
        crossterm::event::KeyCode::Char('t') => Some(MacroCommand::Swap),
        _ => None,
    }
}
