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
            key.and_then(|key| match repeat {
                Repeat::EndCall
                    if key.code == crossterm::event::KeyCode::Char('e')
                        && key.modifiers.is_empty() =>
                {
                    Some(MacroCommand::EndAndCall)
                }
                Repeat::Ring
                    if key.code == crossterm::event::KeyCode::Char('k')
                        && key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                {
                    Some(MacroCommand::CallRepeat)
                }
                Repeat::Ring
                    if key.code == crossterm::event::KeyCode::Char('l')
                        && key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                {
                    Some(MacroCommand::CallSecond)
                }
                Repeat::Ring
                    if key.code == crossterm::event::KeyCode::Char('n')
                        && key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                {
                    Some(MacroCommand::Next)
                }
                Repeat::Ring
                    if key.code == crossterm::event::KeyCode::Char('p')
                        && key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                {
                    Some(MacroCommand::Previous)
                }
                Repeat::Ring
                    if key.code == crossterm::event::KeyCode::Char('d')
                        && key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                {
                    Some(MacroCommand::Delete)
                }
                Repeat::Ring
                    if key.code == crossterm::event::KeyCode::Char('t')
                        && key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                {
                    Some(MacroCommand::Swap)
                }
                _ => None,
            })
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
