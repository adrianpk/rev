use crate::command::WindowCommand;
use crate::input::Focus;
use crate::view::{Error, Size};
use crate::window::{Split, ViewId, Windows};

use super::{Action, Session};

impl Session {
    pub fn windows(&self) -> &Windows {
        &self.windows
    }

    /// Native view selection shares normal command continuity and validates the
    /// live destination. A transient interaction retains its own input focus.
    pub fn select_view(&mut self, id: ViewId) -> Result<(), Error> {
        self.cancel_pointer_capture();
        self.yank = None;
        if self.tangling.is_some()
            || self.execution.is_some()
            || self.capture_blocks_input()
            || self.input.focus() != Focus::Editing
            || self.tree.focused()
        {
            return Err(crate::window::Error::Focus.into());
        }
        self.select_editing_view(id)
    }

    pub(super) fn select_editing_view(&mut self, id: ViewId) -> Result<(), Error> {
        let view = self.windows.get(id)?;
        view.cell(self.buffers.get(view.point.stamp.buffer)?)?;
        self.stop_typing()?;
        self.kills.break_chain();
        self.windows.select(id)?;
        self.sync_input_context();
        Ok(())
    }

    pub(super) fn split(&mut self, direction: Split) -> Action {
        match self.windows.split(direction, self.editing_size()) {
            Ok(_) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    pub(super) fn window_command(&mut self, command: WindowCommand) -> Action {
        if self.input.focus() != Focus::Editing || self.tree.focused() {
            return self.message(&crate::window::Error::Focus.to_string());
        }
        match command {
            WindowCommand::Other => self.cycle_view(),
            WindowCommand::Delete => match self.windows.delete_selected() {
                Ok(_) => self.message(""),
                Err(error) => self.error(&error.to_string()),
            },
            WindowCommand::Keep => {
                if self.windows.delete_others() == 0 {
                    return self.message("Only editing window");
                }
                self.message("")
            }
        }
    }

    fn cycle_view(&mut self) -> Action {
        let result = self
            .windows
            .next_view()
            .map_err(Error::from)
            .and_then(|id| self.select_view(id));
        match result {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    pub(super) fn pane_size(&self) -> Size {
        self.windows
            .area(self.windows.selected_id(), self.editing_size())
            .map(|area| area.view_size())
            .unwrap_or(Size {
                width: 0,
                height: 0,
            })
    }
}
