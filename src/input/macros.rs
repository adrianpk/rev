use super::{Context, Error, Input, Key, Outcome};
use crossterm::event::KeyCode;

pub(crate) const QUESTION: &str = "Macro query: SPC/y continue, DEL/n skip, RET/q stop, C-r edit";

/// Native query answers belong to the current prompt, never to its recording.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacroQuery {
    Continue,
    Skip,
    Stop,
    Edit,
}

impl Input {
    pub(crate) fn macro_query(&mut self) -> Result<(), Error> {
        self.available()?;
        self.context = Context::MacroQuery;
        self.clear_prefix();
        Ok(())
    }
}
pub(super) fn answer(key: Key) -> Outcome {
    let choice = if key.control('r') {
        Some(MacroQuery::Edit)
    } else {
        match key.plain() {
            Some(KeyCode::Char(' ' | 'y')) => Some(MacroQuery::Continue),
            Some(KeyCode::Backspace | KeyCode::Char('n')) => Some(MacroQuery::Skip),
            Some(KeyCode::Enter | KeyCode::Char('q')) => Some(MacroQuery::Stop),
            _ => None,
        }
    };
    choice.map_or(
        Outcome::Message("Answer SPC/y, DEL/n, RET/q or C-r"),
        Outcome::MacroQuery,
    )
}
