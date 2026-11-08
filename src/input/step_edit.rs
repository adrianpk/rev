use super::{Context, Error, Input, Key, Outcome};
use crossterm::event::KeyCode;

/// Role-qualified choices of the active step editor, never standalone commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacroStep {
    Execute,
    Remove,
    Skip,
    Group,
    Continue,
    RemoveRest,
    Cancel,
    Help,
    Insert,
    InsertKey,
    Replace,
    ReplaceKey,
    After,
    Append,
}
impl Input {
    pub(crate) fn macro_step(&mut self, label: String) -> Result<(), Error> {
        self.available()?;
        self.context = Context::MacroStep(label);
        self.clear_prefix();
        Ok(())
    }
    pub(crate) fn macro_input_pending(&self) -> bool {
        self.navigation_pending()
            || self.prefix_argument().supplied()
            || self.prefix().is_some()
            || !self.chords.is_empty()
    }
}
pub(super) fn answer(key: Key) -> Outcome {
    let choice = if key.control('k') {
        Some(MacroStep::RemoveRest)
    } else {
        match key.plain() {
            Some(KeyCode::Char(' ' | 'y')) => Some(MacroStep::Execute),
            Some(KeyCode::Char('n' | 'd') | KeyCode::Backspace) => Some(MacroStep::Remove),
            Some(KeyCode::Char('f')) => Some(MacroStep::Skip),
            Some(KeyCode::Tab) => Some(MacroStep::Group),
            Some(KeyCode::Char('c')) => Some(MacroStep::Continue),
            Some(KeyCode::Char('q')) => Some(MacroStep::Cancel),
            Some(KeyCode::Char('?')) => Some(MacroStep::Help),
            Some(KeyCode::Char('i')) => Some(MacroStep::Insert),
            Some(KeyCode::Char('I')) => Some(MacroStep::InsertKey),
            Some(KeyCode::Char('r')) => Some(MacroStep::Replace),
            Some(KeyCode::Char('R')) => Some(MacroStep::ReplaceKey),
            Some(KeyCode::Char('a')) => Some(MacroStep::After),
            Some(KeyCode::Char('A')) => Some(MacroStep::Append),
            _ => None,
        }
    };
    choice.map_or(
        Outcome::Message("Step editor: ? shows choices"),
        Outcome::MacroStep,
    )
}
