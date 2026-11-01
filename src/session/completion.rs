pub(super) mod resolve;
mod sources;

use crate::buffer::edit::Cursor;
use crate::command::{Command, CompletionCommand as C};
use crate::completion::edits::Edits;
use crate::completion::{Completion, Request};
use crate::input::{Focus, Outcome};

use super::{Action, Session};

impl Session {
    /// Current source-bound candidates. Observation never edits or reparses text.
    pub fn completion(&self) -> Option<&Completion> {
        self.completion.as_ref().filter(|completion| {
            if !self.completion_state_current() {
                return false;
            }
            self.buffer().is_ok_and(|buffer| {
                completion.current(buffer, self.windows.selected_id(), self.view().point)
                    && completion.lsp_current(&self.lsp)
            }) && completion.sources_current(&self.buffers)
                && self.input.focus() == Focus::Editing
                && !self.tree.focused()
        })
    }

    /// Last admitted source outcome for the current invocation, including empty fallback.
    pub fn completion_source(&self) -> Option<crate::completion::CompletionSource> {
        self.completion_source
    }

    pub(super) fn dismiss_completion(&mut self) {
        if let Some(resolve) = self.completion_resolve.take() {
            resolve.cancel();
        }
        self.completion_source = None;
        self.completion = None;
        self.completion_request = None;
        self.completion_state = None;
        self.completion_work.cancel();
        self.input.set_completion(false);
    }

    pub(super) fn invalidate_completion(&mut self) {
        if self.completion_request.is_some() && !self.completion_state_current() {
            self.dismiss_completion();
            return;
        }
        if self.completion.is_some() && self.completion().is_none() {
            self.dismiss_completion();
        }
        if self.completion_request.is_some()
            && self.completion.is_none()
            && !self.resolution_current()
            && !self
                .completion_work
                .current(self.windows.selected_id(), self.view().point)
        {
            self.dismiss_completion();
        }
    }

    pub(super) fn completion_outcome(&mut self, outcome: &Outcome) {
        if matches!(outcome, Outcome::Prefix(_)) && self.input.completion_active() {
            return;
        }
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
            C::Mode | C::AutoMode => self.toggle_completion(command == C::AutoMode),
            C::AtPoint => self.begin_completion(),
            C::Cancel => {
                self.dismiss_completion();
                self.message("Completion cancelled")
            }
            C::Accept => {
                let result = self.publish_completion();
                match result {
                    Ok(true) => self.message(""),
                    Ok(false) => self.message("Resolving completion"),
                    Err(error) => self.error(&error),
                }
            }
            C::Next | C::Previous => self.navigate_completion(command == C::Previous),
        }
    }

    fn begin_completion(&mut self) -> Action {
        self.dismiss_completion();
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        let now = std::time::Instant::now();
        let result = self.capture_completion().map(|mut request| {
            request.due = now;
            self.completion_work.queue(request);
        });
        match result {
            Ok(()) => self.expire_completion(now),
            Err(error) => {
                self.dismiss_completion();
                self.error(&error)
            }
        }
    }

    fn capture_completion(&mut self) -> Result<Request, String> {
        self.activate_lsp();
        if !self.completion_policy.enabled {
            return Err("Completion is disabled".into());
        }
        if !self.completion_focus() {
            return Err("Completion requires source editing focus".into());
        }
        self.completion_generation = self
            .completion_generation
            .checked_add(1)
            .ok_or("Completion identity exhausted")?;
        self.completion_request = Some(self.completion_generation);
        self.completion_state = self.editing_state();
        let point = self.view().point;
        let view = self.windows.selected_id();
        let semantic = self.lsp.completion_possible(point.stamp.buffer);
        let buffer = self
            .buffers
            .get_mut(point.stamp.buffer)
            .map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err("buffer is read-only".into());
        }
        completion_available(buffer, semantic)?;
        let mut request = Request::capture(buffer, view, self.completion_generation, point)?;
        request.policy = self.completion_policy.clone();
        Ok(request)
    }

    pub(super) fn expire_completion(&mut self, now: std::time::Instant) -> Action {
        self.invalidate_completion();
        let mut action = self.poll_resolution(now).unwrap_or(Action::Wait);
        if let Some(result) = self.completion_work.poll(now) {
            action = self.receive_completion(result);
        }
        if self.completion_work.ready(now)
            && let Err(error) = self.launch_completion(now)
        {
            self.dismiss_completion();
            return self.error(&error);
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
        self.prepare_semantic(id)?;
        let (sources, limited) = match self.completion_sources(id) {
            Ok(sources) => sources,
            Err(_)
                if self
                    .completion_work
                    .pending
                    .as_ref()
                    .is_some_and(|r| r.semantic.is_some()) =>
            {
                (Vec::new(), false)
            }
            Err(error) => return Err(error),
        };
        let ids = sources
            .iter()
            .map(|source| source.observation.stamp.buffer)
            .collect();
        if let Some(request) = self.completion_work.pending.as_mut() {
            request.limited = limited;
            request.observations = sources
                .iter()
                .map(|source| source.observation.clone())
                .collect();
        }
        if let Err(error) = self.completion_work.start(sources, now) {
            self.abandon_completion_sources(ids);
            return Err(error);
        }
        Ok(())
    }

    fn prepare_semantic(&mut self, id: crate::buffer::BufferId) -> Result<(), String> {
        self.activate_lsp();
        let buffer = self.buffers.get(id).map_err(|e| e.to_string())?;
        let request = self
            .completion_work
            .pending
            .as_mut()
            .ok_or("No completion pending")?;
        if request.native_only {
            return Ok(());
        }
        request.semantic = self
            .lsp
            .completion(buffer, request.semantic_target(), request.manual);
        if matches!(self.lsp.buffer_status(id), crate::lsp::Status::Ready(caps) if !caps.completion)
        {
            request.fallback_reason = crate::completion::CompletionFallback::Unsupported;
        }
        Ok(())
    }

    fn receive_completion(
        &mut self,
        result: (Request, Result<crate::completion::job::Output, String>),
    ) -> Action {
        let (request, result) = result;
        let output = match result {
            Ok(output) => output,
            Err(error) => return self.failed_completion(request, &error),
        };
        self.restore_completion_sources(output.sources);
        if self.completion_request != Some(output.request.generation) {
            return Action::Redraw;
        }
        self.completion_source = output.result.as_ref().ok().map(Completion::source);
        let manual = output.request.manual;
        let limited = output.request.limited;
        let result = output
            .result
            .and_then(|completion| self.install_completion(completion));
        self.completion_result(result, manual, limited)
    }

    fn completion_result(
        &mut self,
        result: Result<bool, String>,
        manual: bool,
        limited: bool,
    ) -> Action {
        match result {
            Ok(true) if manual && limited => {
                self.message("Native completion limited to admitted observations")
            }
            Ok(true) => self.message(""),
            Ok(false) => {
                self.completion_request = None;
                if manual {
                    self.message(if limited {
                        "Native completion observation limit reached"
                    } else {
                        "No matching native declarations"
                    })
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

    fn failed_completion(&mut self, request: Request, error: &str) -> Action {
        self.abandon_completion_sources(
            request
                .observations
                .iter()
                .map(|source| source.stamp.buffer)
                .collect(),
        );
        if self.completion_request == Some(request.generation) {
            self.dismiss_completion();
        }
        self.error(error)
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
        self.check_completion_target(completion)?;
        if !completion.lsp_current(&self.lsp) {
            return Err("Stale completion target".into());
        }
        Ok(())
    }

    fn check_completion_target(&self, completion: &Completion) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err("buffer is read-only".into());
        }
        if !self.completion_state_current()
            || self.completion_request != Some(completion.generation)
            || !completion.current(buffer, self.windows.selected_id(), self.view().point)
            || !completion.sources_current(&self.buffers)
        {
            return Err("Stale completion target".into());
        }
        if !self.completion_focus() || !self.completion_policy.enabled {
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

    fn publish_completion(&mut self) -> Result<bool, String> {
        if self.completion_resolve.is_some() {
            return Ok(false);
        }
        let completion = self.completion.take().ok_or("No active completion")?;
        self.check_completion(&completion)?;
        if let Some(proposal) = completion.proposal() {
            let ticket = self
                .lsp
                .resolve(proposal)
                .ok_or("Completion connection is obsolete")?;
            self.completion_resolve = Some(resolve::Resolving::new(completion, ticket));
            return Ok(false);
        }
        self.finish_completion(completion).map(|_| true)
    }

    fn finish_completion(&mut self, completion: Completion) -> Result<(), String> {
        self.input.set_completion(false);
        let edit = self.completion_edit(&completion);
        self.completion_request = None;
        self.apply_completion(edit?).map_err(|e| e.to_string())
    }

    fn completion_edit(&self, completion: &Completion) -> Result<Edits, String> {
        self.check_completion(completion)?;
        completion.edits(self.buffer().map_err(|e| e.to_string())?)
    }

    fn apply_completion(&mut self, edit: Edits) -> Result<(), crate::buffer::edit::Error> {
        let buffer = self.buffers.get_mut(edit.0[0].source.stamp.buffer)?;
        let points = self.windows.points(buffer)?;
        let cursor = Cursor {
            index: points.initiator,
            position: edit.1,
        };
        let mut prepared = buffer.prepare(&edit.0, &points.values, Some(cursor))?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.windows.selected_mut().reset_navigation();
        self.published_repeat_boundary();
        self.sync_input_context();
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

impl Session {
    fn toggle_completion(&mut self, automatic: bool) -> Action {
        self.dismiss_completion();
        let value = if automatic {
            &mut self.completion_policy.automatic
        } else {
            &mut self.completion_policy.enabled
        };
        *value = !*value;
        let name = if automatic {
            "Automatic completion"
        } else {
            "Completion"
        };
        let state = if *value { "enabled" } else { "disabled" };
        self.message(&format!("{name} {state}"))
    }

    fn completion_state_current(&self) -> bool {
        self.completion_state == self.editing_state()
    }

    fn automatic_completion_state(&self) -> bool {
        self.editing_state()
            .is_none_or(crate::input::EditingState::text_entry)
    }

    fn completion_focus(&self) -> bool {
        self.input.focus() == Focus::Editing
            && !self.tree.focused()
            && self.pending.is_none()
            && !self.bookmark_list_active()
            && !self.agenda_active()
            && self.habit_agenda.is_none()
            && self.query_agenda.is_none()
    }

    pub(super) fn dispatch_completion_outcome(
        &mut self,
        outcome: Outcome,
        now: std::time::Instant,
    ) -> Action {
        let edited = Self::completion_edit_outcome(&outcome);
        let before = self.view().point.stamp;
        self.completion_outcome(&outcome);
        let action = self.outcome(outcome, now);
        if edited && self.view().point.stamp != before {
            self.schedule_completion(now);
        }
        action
    }

    pub(super) fn completion_edit_outcome(outcome: &Outcome) -> bool {
        matches!(
            outcome,
            Outcome::Editing(_)
                | Outcome::Command(Command::BackwardDeleteChar | Command::DeleteChar)
        )
    }

    fn automatic_completion(&self, request: &Request) -> bool {
        self.buffer().is_ok_and(|buffer| {
            let trigger = request.point.position.0.checked_sub(1).and_then(|at| {
                buffer
                    .slice(&buffer.span(at..at + 1).ok()?)
                    .ok()
                    .map(|text| text.to_string())
            });
            request.automatic(buffer)
                || trigger.is_some_and(|text| self.lsp.triggered(buffer.id(), &text))
        })
    }

    pub(super) fn schedule_completion(&mut self, now: std::time::Instant) {
        if !self.automatic_completion_state()
            || !self.completion_policy.enabled
            || !self.completion_policy.automatic
        {
            return;
        }
        self.dismiss_completion();
        let Ok(mut request) = self.capture_completion() else {
            self.completion_request = None;
            return;
        };
        if !self.automatic_completion(&request) {
            self.completion_request = None;
            return;
        }
        request.manual = false;
        request.due =
            now + std::time::Duration::from_millis(self.completion_policy.delay_ms as u64);
        self.completion_work.queue(request);
    }
}

#[cfg(test)]
mod tests;

fn completion_available(buffer: &crate::buffer::Buffer, semantic: bool) -> Result<(), String> {
    if semantic {
        return Ok(());
    }
    if buffer.len_bytes() > crate::completion::MAX_BYTES {
        return Err("Native completion buffer exceeds 2 MiB".into());
    }
    if !buffer.completion_available() {
        return Err("Native completion syntax is unavailable".into());
    }
    Ok(())
}
