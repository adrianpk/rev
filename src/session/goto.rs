use crate::buffer::Stamp;
use crate::coordinates::{Coordinates, LineByte, Scalar};
use crate::view::Error;
use crate::window::ViewId;

use super::{Action, Pending, Session};

impl Session {
    pub(super) fn goto_line(&mut self) -> Action {
        self.line_prompt(self.windows.selected_id(), self.view().point.stamp, "")
    }

    fn line_prompt(&mut self, view: ViewId, source: Stamp, draft: &str) -> Action {
        if let Err(error) = self.input.argument("Goto line: ", draft) {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::GotoLine { view, source });
        self.message("")
    }

    pub(super) fn goto_argument(&mut self, view: ViewId, source: Stamp, draft: &str) -> Action {
        if view != self.windows.selected_id() || source != self.view().point.stamp {
            return self.message("Goto line source changed");
        }
        let line = match draft.trim().parse::<i64>() {
            Ok(line) => line,
            Err(_) => {
                self.line_prompt(view, source, draft);
                return self.message("Enter a signed 64-bit decimal line number");
            }
        };
        match self.move_to_line(line) {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn move_to_line(&mut self, line: i64) -> Result<(), Error> {
        let buffer = self.buffer()?;
        let mut candidate = self.view().clone();
        candidate.cell(buffer)?;
        let coordinates = Coordinates::new(buffer)?;
        candidate.point.position = if line <= 0 {
            Scalar(0)
        } else if let Ok(row) = usize::try_from(line - 1)
            && row < buffer.len_lines()
        {
            coordinates.from_line(LineByte { row, column: 0 })?
        } else {
            coordinates.end()
        };
        candidate.reset_navigation();
        candidate.reveal(buffer, self.pane_size())?;
        *self.windows.selected_mut() = candidate;
        Ok(())
    }
}
