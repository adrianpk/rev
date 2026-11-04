//! The composer borrows a native transient view; source and terminal owners
//! survive beneath it. Only an explicit submit authorizes the Git child.
use super::*;
use crate::{buffer::BufferId, git::budget::Lease, input::EditingState, window::ViewId};

pub(super) struct Composer {
    observation: Arc<Observation>,
    buffer: BufferId,
    view: ViewId,
    tree: bool,
    terminal: bool,
    editing: Option<EditingState>,
    _admission: Lease,
}

impl Session {
    pub(in crate::session) fn git_composing(&self) -> bool {
        self.git.composer.is_some()
    }

    pub(super) fn commit_command(&mut self, command: Command) -> Option<Action> {
        if !self.git_composing() {
            return None;
        }
        if self.input.focus() != Focus::Editing {
            return None;
        }
        Some(match command {
            Command::Git(GitCommand::CommitSubmit) => self.submit_git_commit(),
            Command::Git(GitCommand::CommitCancel) | Command::KeyboardQuit => {
                self.cancel_git_commit()
            }
            _ if editing(command) => return None,
            _ => self
                .message("Commit composer owns this view; C-Enter submits, C-g retains the draft"),
        })
    }

    pub(super) fn compose_git_commit(&mut self) -> Action {
        match self.install_git_commit() {
            Ok(()) => self.message("Compose commit: Return inserts newline; C-Enter or C-c C-c submits; C-g retains draft"),
            Err(error) => self.error(&error),
        }
    }

    fn install_git_commit(&mut self) -> Result<(), String> {
        if !self.git.visible || self.git.page != Page::Status {
            return Err("Open Git Status before composing a commit".into());
        }
        if self.windows.retained_view_active() || self.layout_active() {
            return Err(
                "Another interaction retains the source view; finish it before composition".into(),
            );
        }
        let context = self
            .git
            .state
            .context
            .ok_or("Git Status is still checking context")?;
        let observation = self.git.engine.commit_observation(context)?;
        // One active native draft: <=64 KiB text, <=16 retained history groups
        // (128 KiB payload target), <=64 ranges/transaction and <=16 points.
        // Admission also covers rope chunks, edit candidates and projection.
        let admission = self.git.engine.source_admission(8 * 1024 * 1024)?;
        let text = self.git.engine.commit_draft(context).unwrap_or("");
        let id = self.buffers.commit_draft(text).map_err(|e| e.to_string())?;
        let buffer = self.buffers.get(id).map_err(|e| e.to_string())?;
        let mut point = crate::view::View::new(buffer).point;
        point.position = crate::coordinates::Scalar(buffer.len_chars());
        if let Err(error) = self.windows.retain_view(buffer, point) {
            self.discard_capture_buffer(id)?;
            return Err(error);
        }
        self.git.composer = Some(Composer {
            observation,
            buffer: id,
            view: self.windows.selected_id(),
            tree: self.tree.focused(),
            terminal: self.terminals.focused,
            editing: self.input.editing_state(),
            _admission: admission,
        });
        self.git.visible = false;
        self.git.state.menu = false;
        self.tree.edit();
        self.terminals.focused = false;
        self.input.cancel();
        self.cancel_pointer_capture();
        self.cancel_terminal_pointer();
        self.sync_input_context();
        self.input.set_editing_state(EditingState::Insert);
        Ok(())
    }

    fn commit_text(&self) -> Result<String, String> {
        let composer = self
            .git
            .composer
            .as_ref()
            .ok_or("No commit composer is active")?;
        if self.windows.selected_id() != composer.view
            || self.view().point.stamp.buffer != composer.buffer
        {
            return Err("Commit composer view changed; draft retained".into());
        }
        let buffer = self
            .buffers
            .get(composer.buffer)
            .map_err(|e| e.to_string())?;
        buffer
            .slice(
                &buffer
                    .span(0..buffer.len_chars())
                    .map_err(|e| e.to_string())?,
            )
            .map(|s| s.to_string())
            .map_err(|e| e.to_string())
    }

    fn cancel_git_commit(&mut self) -> Action {
        let result = self.commit_text().and_then(|text| {
            let context = self
                .git
                .composer
                .as_ref()
                .expect("composer")
                .observation
                .context;
            self.git.engine.retain_commit_draft(context, &text)?;
            self.leave_git_commit()
        });
        match result {
            Ok(()) => self.message("Commit cancelled before submission; session draft retained"),
            Err(error) => self.error(&error),
        }
    }

    fn submit_git_commit(&mut self) -> Action {
        let result = self.commit_text().and_then(|text| {
            let observation = self
                .git
                .composer
                .as_ref()
                .expect("composer")
                .observation
                .clone();
            let operation = self.git.engine.commit(observation, &text)?;
            self.git.commit_request = Some(operation);
            self.leave_git_commit()
        });
        match result {
            Ok(()) => self.message(
                "Committing checked index; draft retained until verification; $ inspects process",
            ),
            Err(error) => self.error(&error),
        }
    }

    fn leave_git_commit(&mut self) -> Result<(), String> {
        let state = self
            .git
            .composer
            .take()
            .ok_or("No commit composer is active")?;
        self.windows.restore_view();
        self.discard_capture_buffer(state.buffer)?;
        self.windows.select(state.view).map_err(|e| e.to_string())?;
        if state.tree {
            self.tree.focus(self.size).map_err(|e| e.to_string())?;
        }
        self.terminals.focused = state.terminal;
        self.input.cancel();
        self.input
            .editing_owner(self.windows.selected_id(), self.view().point.stamp.buffer);
        if let Some(editing) = state.editing {
            self.input.set_editing_state(editing);
        }
        self.git.visible = true;
        self.git.page = Page::Status;
        self.sync_input_context();
        Ok(())
    }

    pub(super) fn publish_git_commit(&mut self, update: &crate::git::Update) {
        let verified = self
            .git
            .engine
            .records()
            .iter()
            .find(|r| r.id == update.operation)
            .is_some_and(|r| r.outcome == crate::git::Outcome::Verified);
        if self
            .git
            .engine
            .progress()
            .all(|p| p.operation != update.operation)
        {
            self.git.commit_request = None;
        }
        if update.published {
            self.git.state.context = update.context;
            self.git.inspection = comparisons::Inspection::default();
            self.git.reconcile(self.git_extent());
        }
        if self.git.visible && self.input.focus() == Focus::Editing && self.pending.is_none() {
            self.message(if verified { "Commit created and verified" } else {
                "Commit failed or effects uncertain; session draft retained; $ inspects actual diagnostics"
            });
        }
    }
}

fn editing(command: Command) -> bool {
    matches!(
        command,
        Command::Motion(_)
            | Command::Scroll(_)
            | Command::Newline
            | Command::InsertTab
            | Command::BackwardDeleteChar
            | Command::DeleteChar
            | Command::History(_)
            | Command::Region(_)
            | Command::Rectangle(_)
            | Command::Kill(_)
            | Command::Linear(_)
            | Command::SearchForward
            | Command::SearchBackward
            | Command::QueryReplace
            | Command::GotoLine
            | Command::ExecuteExtended
    )
}
