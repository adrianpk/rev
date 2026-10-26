use crate::command::RectangleCommand;
use crate::input::Focus;
use crate::rectangle::{Rectangle, Selection};

use super::{Action, Session};

impl Session {
    pub fn rectangle(&self) -> Option<&Rectangle> {
        self.rectangle.as_ref()
    }

    pub(super) fn rectangle_command(&mut self, command: RectangleCommand) -> Action {
        if self.input.focus() != Focus::Editing {
            return self.message("a prompt owns input focus");
        }
        let result = match command {
            RectangleCommand::Mark => self.rectangle_mark(),
            RectangleCommand::Copy => self.copy_rectangle(),
        };
        match result {
            Ok(message) => self.message(message),
            Err(error) => self.message(&error.to_string()),
        }
    }

    fn rectangle_mark(&mut self) -> Result<&'static str, crate::buffer::edit::Error> {
        let point = self.view().point;
        let buffer = self.buffers.get_mut(point.stamp.buffer)?;
        buffer.toggle_rectangle(point)?;
        Ok(if buffer.rectangle_active() {
            "Rectangle mark set"
        } else {
            "Rectangle mark off"
        })
    }

    fn copy_rectangle(&mut self) -> Result<&'static str, crate::buffer::edit::Error> {
        let buffer = self.buffer().map_err(crate::buffer::edit::Error::from)?;
        let Some(selection) = Selection::new(buffer, self.view().point)? else {
            return Ok("Rectangle is not active");
        };
        if selection.width() == 0 {
            return Ok("Rectangle is empty");
        }
        let rectangle = Rectangle::copy(buffer, &selection)?;
        self.rectangle = Some(rectangle);
        Ok("Rectangle copied")
    }
}
