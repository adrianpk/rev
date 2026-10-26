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
        if self.input.focus() != Focus::Editing {
            return Err(crate::window::Error::Focus.into());
        }
        let view = self.windows.get(id)?;
        view.cell(self.buffers.get(view.point.stamp.buffer)?)?;
        self.stop_typing()?;
        self.windows.select(id)?;
        Ok(())
    }

    pub(super) fn split(&mut self, direction: Split) -> Action {
        match self.windows.split(direction, self.size) {
            Ok(_) => self.message(""),
            Err(error) => self.message(&error.to_string()),
        }
    }

    pub(super) fn pane_size(&self) -> Size {
        self.windows
            .area(self.windows.selected_id(), self.size)
            .map(|area| area.view_size())
            .unwrap_or(Size {
                width: 0,
                height: 0,
            })
    }
}
