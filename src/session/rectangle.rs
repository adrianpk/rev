use crate::command::RectangleCommand;
use crate::input::Focus;
use crate::rectangle::{Capture, Rectangle, Selection};

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
            RectangleCommand::Copy => self.retain_rectangle(false),
            RectangleCommand::Kill => self.retain_rectangle(true),
            RectangleCommand::Yank => self.yank_rectangle(),
        };
        match result {
            Ok(message) => self.message(message),
            Err(error) => self.message(&error.to_string()),
        }
    }

    fn yank_rectangle(&mut self) -> Result<&'static str, crate::buffer::edit::Error> {
        let Some(rectangle) = &self.rectangle else {
            return Ok("No rectangle to yank");
        };
        let point = self.windows.selected().point;
        let buffer = self.buffers.get_mut(point.stamp.buffer)?;
        let insertion = rectangle.insertions(buffer, point)?;
        let points = self.windows.points(buffer)?;
        let cursor = crate::buffer::edit::Cursor {
            index: points.initiator,
            position: insertion.cursor,
        };
        let mut prepared = buffer.prepare(&insertion.edits, &points.values, Some(cursor))?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.windows.selected_mut().reset_navigation();
        Ok("Rectangle yanked")
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

    fn retain_rectangle(&mut self, kill: bool) -> Result<&'static str, crate::buffer::edit::Error> {
        let buffer = self.buffer().map_err(crate::buffer::edit::Error::from)?;
        let Some(selection) = Selection::new(buffer, self.view().point)? else {
            return Ok("Rectangle is not active");
        };
        if selection.width() == 0 {
            return Ok("Rectangle is empty");
        }
        let capture = if kill { Capture::Kill } else { Capture::Copy };
        let rectangle = Rectangle::capture(buffer, &selection, capture)?;
        if kill {
            self.remove_rectangle(&selection)?;
        }
        self.rectangle = Some(rectangle);
        Ok(if kill {
            "Rectangle killed"
        } else {
            "Rectangle copied"
        })
    }
    fn remove_rectangle(
        &mut self,
        selection: &Selection,
    ) -> Result<(), crate::buffer::edit::Error> {
        let buffer = self
            .buffers
            .get_mut(self.windows.selected().point.stamp.buffer)?;
        let edits = selection.deletions(buffer)?;
        let points = self.windows.points(buffer)?;
        let cursor = crate::buffer::edit::Cursor {
            index: points.initiator,
            position: selection.cursor(buffer)?,
        };
        let mut prepared = buffer.prepare(&edits, &points.values, Some(cursor))?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.windows.selected_mut().reset_navigation();
        Ok(())
    }
}
