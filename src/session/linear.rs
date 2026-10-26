use std::fmt;

use crate::buffer::{Span, edit};
use crate::command::LinearCommand;
use crate::input::Focus;

use super::{Action, Session};

enum Error {
    Edit(edit::Error),
    Inactive,
    EmptyRing,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => error.fmt(formatter),
            Self::Inactive => formatter.write_str("Region is not active"),
            Self::EmptyRing => formatter.write_str("Kill ring is empty"),
        }
    }
}

impl From<edit::Error> for Error {
    fn from(error: edit::Error) -> Self {
        Self::Edit(error)
    }
}

impl Session {
    pub(super) fn linear_command(&mut self, command: LinearCommand) -> Action {
        if self.input.focus() != Focus::Editing {
            self.kills.break_chain();
            return self.message("a prompt owns input focus");
        }
        let result = self.linear_edit(command);
        if !matches!(result, Ok(true)) {
            self.kills.break_chain();
        }
        match result {
            Ok(true) if command == LinearCommand::Copy => self.message("Region copied"),
            Ok(changed) => self.finish(Ok(changed)),
            Err(error) => self.message(&error.to_string()),
        }
    }

    fn linear_edit(&mut self, command: LinearCommand) -> Result<bool, Error> {
        match command {
            LinearCommand::Kill => {
                let source = self.active_region()?;
                let backward = self.windows.selected().point.position.0 == source.range.end;
                Ok(self.kill_source(source, backward)?)
            }
            LinearCommand::Copy => {
                let source = self.active_region()?;
                Ok(self.copy_region(source)?)
            }
            LinearCommand::Yank => self.yank(),
        }
    }

    fn active_region(&self) -> Result<Span, Error> {
        self.buffer()
            .map_err(edit::Error::from)?
            .region(self.windows.selected().point)?
            .ok_or(Error::Inactive)
    }

    fn copy_region(&mut self, source: Span) -> Result<bool, edit::Error> {
        if source.range.is_empty() {
            return Ok(false);
        }
        let text = self.buffer()?.slice(&source)?.into();
        self.kills.copy(text);
        Ok(true)
    }

    fn yank(&mut self) -> Result<bool, Error> {
        let text = self.kills.text().ok_or(Error::EmptyRing)?.to_string();
        let point = self.windows.selected().point.position.0;
        Ok(self.replace(point..point, &text)?)
    }
}
