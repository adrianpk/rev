use crate::buffer::edit::{Cursor, Replacement};
use crate::command::{Command, CompletionCommand as C};
use crate::completion::{Completion, Request};
use crate::coordinates::Scalar;
use crate::input::{Focus, Outcome};

use super::{Action, Session};

impl Session {
    /// Current source-bound candidates. Observation never edits or reparses text.
    pub fn completion(&self) -> Option<&Completion> {
        self.completion.as_ref().filter(|completion| {
            self.buffer().is_ok_and(|buffer| {
                completion.current(buffer, self.windows.selected_id(), self.view().point)
            }) && self.input.focus() == Focus::Editing
                && !self.tree.focused()
        })
    }

    pub(super) fn dismiss_completion(&mut self) {
        self.completion = None;
        self.completion_request = None;
        self.completion_work.cancel();
        self.input.set_completion(false);
    }

    pub(super) fn invalidate_completion(&mut self) {
        if self.completion.is_some() && self.completion().is_none() {
            self.dismiss_completion();
        }
        if self.completion_request.is_some()
            && self.completion.is_none()
            && !self
                .completion_work
                .current(self.windows.selected_id(), self.view().point)
        {
            self.dismiss_completion();
        }
    }

    pub(super) fn completion_outcome(&mut self, outcome: &Outcome) {
        if !matches!(
            outcome,
            Outcome::Command(Command::Completion(_)) | Outcome::Ignored
        ) {
            self.dismiss_completion();
        }
    }

    pub(super) fn completion_boundary(&mut self, command: Command) -> Option<Action> {
        let Command::Completion(command) = command else {
            self.dismiss_completion();
            return None;
        };
        Some(self.completion_command(command))
    }

    fn completion_command(&mut self, command: C) -> Action {
        match command {
            C::AtPoint => self.begin_completion(),
            C::Cancel => {
                self.dismiss_completion();
                self.message("Completion cancelled")
            }
            C::Accept => {
                let result = self.publish_completion();
                match result {
                    Ok(()) => self.message(""),
                    Err(error) => self.error(&error),
                }
            }
            C::Next | C::Previous => self.navigate_completion(command == C::Previous),
        }
    }

    fn begin_completion(&mut self) -> Action {
        self.dismiss_completion();
        let now = std::time::Instant::now();
        let result = self
            .capture_completion()
            .map(|request| self.completion_work.queue(request));
        match result {
            Ok(()) => self.expire_completion(now),
            Err(error) => {
                self.dismiss_completion();
                self.error(&error)
            }
        }
    }

    fn capture_completion(&mut self) -> Result<Request, String> {
        if self.input.focus() != Focus::Editing || self.tree.focused() || self.pending.is_some() {
            return Err("Completion requires source editing focus".into());
        }
        self.stop_typing().map_err(|e| e.to_string())?;
        self.completion_generation = self
            .completion_generation
            .checked_add(1)
            .ok_or("Completion identity exhausted")?;
        self.completion_request = Some(self.completion_generation);
        let point = self.view().point;
        let view = self.windows.selected_id();
        let buffer = self
            .buffers
            .get_mut(point.stamp.buffer)
            .map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err("buffer is read-only".into());
        }
        if buffer.len_bytes() > crate::completion::MAX_BYTES {
            return Err("Native completion buffer exceeds 2 MiB".into());
        }
        Request::capture(buffer, view, self.completion_generation, point)
    }

    pub(super) fn expire_completion(&mut self, now: std::time::Instant) -> Action {
        let mut action = Action::Wait;
        if let Some(result) = self.completion_work.poll(now) {
            action = self.receive_completion(result);
        }
        if self.completion_work.ready(now) {
            if let Err(error) = self.launch_completion(now) {
                self.dismiss_completion();
                return self.error(&error);
            }
        }
        action
    }

    fn launch_completion(&mut self, now: std::time::Instant) -> Result<(), String> {
        let id = self
            .completion_work
            .pending
            .as_ref()
            .ok_or("No completion pending")?
            .point
            .stamp
            .buffer;
        let snapshot = self
            .buffers
            .get_mut(id)
            .map_err(|e| e.to_string())?
            .completion_snapshot()?;
        if let Err(error) = self.completion_work.start(snapshot, now) {
            self.buffers
                .get_mut(id)
                .map_err(|e| e.to_string())?
                .abandon_completion_syntax();
            return Err(error);
        }
        Ok(())
    }

    fn receive_completion(
        &mut self,
        result: Result<crate::completion::job::Output, String>,
    ) -> Action {
        let output = match result {
            Ok(output) => output,
            Err(error) => return self.error(&error),
        };
        if let Ok(buffer) = self.buffers.get_mut(output.request.point.stamp.buffer) {
            buffer.restore_completion_syntax(output.syntax);
        }
        if self.completion_request != Some(output.request.generation) {
            return Action::Redraw;
        }
        let manual = output.request.manual;
        let result = output
            .result
            .and_then(|symbols| self.install_completion(output.request.complete(symbols)));
        match result {
            Ok(true) => self.message(""),
            Ok(false) => {
                self.completion_request = None;
                if manual {
                    self.message("No matching native declarations")
                } else {
                    Action::Redraw
                }
            }
            Err(error) => {
                self.completion_request = None;
                if manual {
                    self.error(&error)
                } else {
                    Action::Redraw
                }
            }
        }
    }

    fn install_completion(&mut self, completion: Completion) -> Result<bool, String> {
        self.check_completion(&completion)?;
        if completion.candidates().is_empty() {
            return Ok(false);
        }
        self.completion = Some(completion);
        self.input.set_completion(true);
        Ok(true)
    }

    fn check_completion(&self, completion: &Completion) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if self.completion_request != Some(completion.generation)
            || !completion.current(buffer, self.windows.selected_id(), self.view().point)
        {
            return Err("Stale completion target".into());
        }
        if self.input.focus() != Focus::Editing || self.tree.focused() || self.pending.is_some() {
            return Err("Completion requires source editing focus".into());
        }
        Ok(())
    }

    fn navigate_completion(&mut self, previous: bool) -> Action {
        self.invalidate_completion();
        let Some(completion) = &mut self.completion else {
            return self.message("No active completion");
        };
        completion.navigate(previous);
        self.message("")
    }

    fn publish_completion(&mut self) -> Result<(), String> {
        let completion = self.completion.take().ok_or("No active completion")?;
        self.input.set_completion(false);
        let edit = self.completion_edit(&completion);
        self.completion_request = None;
        self.apply_completion(edit?).map_err(|e| e.to_string())
    }

    fn completion_edit(&self, completion: &Completion) -> Result<Replacement, String> {
        self.check_completion(completion)?;
        completion.replacement(self.buffer().map_err(|e| e.to_string())?)
    }

    fn apply_completion(&mut self, edit: Replacement) -> Result<(), crate::buffer::edit::Error> {
        let buffer = self.buffers.get_mut(edit.source.stamp.buffer)?;
        let points = self.windows.points(buffer)?;
        let cursor = Cursor {
            index: points.initiator,
            position: Scalar(edit.source.range.start + edit.text.chars().count()),
        };
        let mut prepared = buffer.prepare(&[edit], &points.values, Some(cursor))?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.windows.selected_mut().reset_navigation();
        self.yank = None;
        self.kills.break_chain();
        Ok(())
    }

    pub(super) fn completion_frame(
        &self,
        frame: &mut crate::display::Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), crate::view::Error> {
        let Some(completion) = self.completion() else {
            return Ok(());
        };
        let panel = composition.and_then(|composition| {
            composition
                .panels()
                .iter()
                .find(|(id, _)| *id == self.windows.selected_id())
        });
        if let Some((_, panel)) = panel {
            frame.native_completion(completion, panel.content())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
