use crate::buffer::{Span, edit::Error};
use crate::command::KillCommand;
use crate::coordinates::Coordinates;
use crate::input::Focus;

use super::{Action, Session};

impl Session {
    /// The newest copied entry or directional kill chain.
    pub fn killed_text(&self) -> Option<&ropey::Rope> {
        self.kills.text()
    }

    /// Borrow the session's at most 32 owned linear entries, newest first.
    pub fn kill_ring(&self) -> impl DoubleEndedIterator<Item = &ropey::Rope> + ExactSizeIterator {
        self.kills.iter()
    }

    pub(super) fn kill_command(&mut self, command: KillCommand) -> Action {
        if self.input.focus() != Focus::Editing {
            self.kills.break_chain();
            return self.message("a prompt owns input focus");
        }
        let result = self.kill(command);
        if !matches!(result, Ok(true)) {
            self.kills.break_chain();
        }
        self.finish(result)
    }

    fn kill(&mut self, command: KillCommand) -> Result<bool, Error> {
        let source = self.kill_span(command)?;
        self.kill_source(source, command == KillCommand::BackwardWord)
    }

    pub(super) fn kill_source(&mut self, source: Span, backward: bool) -> Result<bool, Error> {
        if source.range.is_empty() {
            return Ok(false);
        }
        let id = self.windows.selected_id();
        let buffer = self.buffer()?;
        let text = self.kills.prepare(buffer, id, &source, backward)?;
        self.replace(source.range, "")?;
        let point = self.windows.selected().point;
        self.kills.publish(text, point.stamp, id, point.position);
        self.windows.selected_mut().reset_navigation();
        Ok(true)
    }

    fn kill_span(&self, command: KillCommand) -> Result<Span, Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let range = command.range(&coordinates, self.windows.selected().point.position)?;
        Ok(buffer.span(range)?)
    }
}
