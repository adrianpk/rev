mod direct;
mod lines;
mod range;
pub(super) mod transfer;

use super::{Action, Session};
use crate::command::editing::Action as EditAction;
use crate::command::{Command, Editing};
use crate::coordinates::Scalar;
use crate::kill::Shape;

#[derive(Debug)]
pub(super) enum Error {
    Edit(crate::buffer::edit::Error),
    View(crate::view::Error),
    Search(crate::search::Error),
    Target,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Edit(e) => e.fmt(f),
            Self::View(e) => e.fmt(f),
            Self::Search(e) => e.fmt(f),
            Self::Target => f.write_str("Operator target is unavailable"),
        }
    }
}
impl From<crate::buffer::edit::Error> for Error {
    fn from(e: crate::buffer::edit::Error) -> Self {
        Self::Edit(e)
    }
}
impl From<crate::buffer::Error> for Error {
    fn from(e: crate::buffer::Error) -> Self {
        Self::Edit(e.into())
    }
}
impl From<crate::coordinates::Error> for Error {
    fn from(e: crate::coordinates::Error) -> Self {
        Self::Edit(e.into())
    }
}
impl From<crate::view::Error> for Error {
    fn from(e: crate::view::Error) -> Self {
        Self::View(e)
    }
}
impl From<crate::search::Error> for Error {
    fn from(e: crate::search::Error) -> Self {
        Self::Search(e)
    }
}

pub(super) struct Selection {
    pub(super) source: crate::buffer::Span,
    pub(super) shape: Shape,
    pub(super) backward: bool,
}

impl Session {
    pub(super) fn editing_dispatch(&mut self, command: Command) -> Option<Action> {
        let Command::Edit(request) = command else {
            return None;
        };
        if let Some(action) = self.visual_editing(request) {
            return Some(action);
        }
        if request.pending() {
            if self.editing_state() != Some(crate::input::EditingState::Normal) {
                return Some(self.message("Operator composition requires the modal Normal state"));
            }
            return Some(self.begin_operator(request));
        }
        if request.action == EditAction::Replace && request.character.is_none() {
            return Some(self.begin_replacement(request));
        }
        if let Some(action) = self.operator_search(request) {
            return Some(action);
        }
        let result = self.apply_editing(request);
        Some(self.finish_editing(request, result))
    }

    fn finish_editing(&mut self, request: Editing, result: Result<bool, Error>) -> Action {
        if !matches!(result, Ok(true)) {
            self.kills.break_chain();
        }
        match result {
            Ok(_) if request.action.copies() => self.message("Target copied"),
            Ok(changed) => self.finish(Ok(changed)),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn begin_operator(&mut self, request: Editing) -> Action {
        let outcome = self.input.expect_operator(request);
        self.outcome(outcome, std::time::Instant::now())
    }

    fn begin_replacement(&mut self, request: Editing) -> Action {
        let outcome = self.input.expect_replacement(request);
        self.outcome(outcome, std::time::Instant::now())
    }

    pub(super) fn apply_editing(&mut self, request: Editing) -> Result<bool, Error> {
        if !request.action.copies() && self.buffer()?.is_readonly() {
            return Err(crate::buffer::edit::Error::ReadOnly.into());
        }
        if request.action.operator()
            || matches!(
                request.action,
                EditAction::DeleteEnd | EditAction::ChangeEnd | EditAction::CopyEnd
            )
        {
            return self.apply_operator(request);
        }
        self.direct_edit(request)
    }

    fn apply_operator(&mut self, request: Editing) -> Result<bool, Error> {
        let selection = self.editing_range(request)?.ok_or(Error::Target)?;
        self.apply_selection(request, selection)
    }

    pub(super) fn apply_selection(
        &mut self,
        request: Editing,
        selection: Selection,
    ) -> Result<bool, Error> {
        if request.action.shifts() {
            return self.shift_lines(&selection.source, request.action);
        }
        if selection.source.range.is_empty() {
            if request.action.changes() {
                self.input
                    .set_editing_state(crate::input::EditingState::Insert);
            }
            return Ok(false);
        }
        let transfer = crate::kill::Transfer {
            view: self.windows.selected_id(),
            backward: selection.backward,
            shape: selection.shape,
        };
        let entry = self
            .kills
            .prepare_transfer(self.buffer()?, &selection.source, transfer)?;
        if request.action.copies() {
            self.kills.publish_transfer(entry);
            return Ok(false);
        }
        self.remove_operator(request, selection, entry)
    }

    fn remove_operator(
        &mut self,
        request: Editing,
        selection: Selection,
        entry: crate::kill::Entry,
    ) -> Result<bool, Error> {
        let replacement = self.change_line_text(request, &selection)?;
        let cursor = Scalar(selection.source.range.start);
        let changed = self.replace_cursor(selection.source.range, &replacement, cursor)?;
        let point = self.view().point;
        self.kills.publish(
            entry,
            point.stamp,
            self.windows.selected_id(),
            point.position,
        );
        if request.action.changes() {
            self.input
                .set_editing_state(crate::input::EditingState::Insert);
        }
        Ok(changed)
    }

    pub(super) fn change_line_text(
        &self,
        request: Editing,
        selection: &Selection,
    ) -> Result<String, Error> {
        if request.action.changes() && selection.shape == Shape::Line {
            let buffer = self.buffer()?;
            let source = buffer.slice(&selection.source)?;
            if source.chars().last() == Some('\n') {
                return Ok(buffer.newline().into());
            }
        }
        Ok(String::new())
    }

    pub(super) fn editing_preflight(&self, request: Editing) -> Result<(), crate::view::Error> {
        let result = if request.action == EditAction::Repeat {
            self.repeat_preflight(request.count)
        } else {
            self.editing_range(request).map(|_| ())
        };
        match result {
            Err(Error::View(error)) if error.pending() => Err(error),
            Err(Error::Edit(crate::buffer::edit::Error::Coordinates(
                crate::coordinates::Error::Pending,
            ))) => Err(crate::coordinates::Error::Pending.into()),
            _ => Ok(()),
        }
    }
}
