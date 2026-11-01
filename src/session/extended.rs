use crate::buffer::{Visual, edit::Point};
use crate::input::EditingState;
use crate::window::ViewId;

pub(super) struct Origin {
    view: ViewId,
    point: Point,
    mark: Option<crate::coordinates::Scalar>,
    shape: Visual,
}

use super::{Action, Pending, Session};

impl Session {
    pub(super) fn extended(&mut self) -> Action {
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        let origin = self.visual_command_origin();
        if let Err(error) = self.input.commands() {
            return self.error(&error.to_string());
        }
        // Entry is transient input; the eventual shared handler owns command
        // continuity, so a named yank-pop can consume the current yank capture.
        self.pending = Some(Pending::Command);
        self.command_origin = origin;
        self.message("")
    }

    pub(super) fn input_changed(&mut self) -> Action {
        if let Some(action) = self.refresh_timestamp() {
            return action;
        }
        if self.input.focus() == crate::input::Focus::Search {
            return self.search_step();
        }
        if self.input.prompt().is_some() {
            self.echo.clear();
        }
        self.sync_path();
        Action::Redraw
    }
}

impl Session {
    fn visual_command_origin(&self) -> Option<Origin> {
        let shape = self.editing_state()?.visual()?;
        let buffer = self.buffer().ok()?;
        if buffer.visual() != Some(shape) || self.visual_source != Some(buffer.stamp()) {
            return None;
        }
        Some(Origin {
            view: self.windows.selected_id(),
            point: self.view().point,
            mark: buffer.mark(),
            shape,
        })
    }

    pub(super) fn cancel_extended(&mut self, command: crate::command::Command) -> Option<Action> {
        if command != crate::command::Command::KeyboardQuit
            || !matches!(self.pending, Some(Pending::Command))
        {
            return None;
        }
        self.pending = None;
        let Some(origin) = self.command_origin.take() else {
            return Some(self.keyboard_quit());
        };
        self.input.cancel();
        if origin.current(self) {
            // The native range stayed authoritative while the selector owned
            // input. Cancellation retains it; no snapshot is copied back.
            self.message("Quit");
        } else {
            self.leave_visual();
            self.input.set_editing_state(EditingState::Normal);
            self.message("Command source changed; Visual selection retired");
        }
        Some(Action::Redraw)
    }

    pub(super) fn retire_command_origin(&mut self) {
        if let Some(origin) = self.command_origin.take()
            && !origin.current(self)
        {
            self.leave_visual();
            self.input.set_editing_state(EditingState::Normal);
        }
    }
}

impl Origin {
    fn current(&self, session: &Session) -> bool {
        self.view == session.windows.selected_id()
            && self.point == session.view().point
            && session.editing_state() == Some(EditingState::from_visual(self.shape))
            && session.buffer().is_ok_and(|buffer| {
                buffer.visual() == Some(self.shape) && buffer.mark() == self.mark
            })
    }
}
