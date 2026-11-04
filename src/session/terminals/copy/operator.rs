use super::State;
use crate::{
    buffer::Span,
    command::{
        Editing, Navigation, Scroll,
        editing::{Action, Object, Operand},
        navigation::Target,
    },
    coordinates::{Cell, Coordinates, Scalar},
    kill::Shape,
    view::{Error, Size},
};
use std::ops::Range;

impl State {
    pub fn operator(&self, request: Editing, size: Size) -> Result<(Span, Shape), String> {
        let (range, shape) = self
            .operator_range(request, size)
            .map_err(|e| e.to_string())?;
        Ok((self.buffer.span(range).map_err(|e| e.to_string())?, shape))
    }
    fn operator_range(&self, request: Editing, size: Size) -> Result<(Range<usize>, Shape), Error> {
        let operand = request.operand.or_else(|| end_operand(request));
        match operand {
            Some(Operand::Lines) => {
                let mut view = self.view.clone();
                view.scroll(
                    &self.buffer,
                    Scroll::NextLine,
                    request.amount().saturating_sub(1),
                )?;
                Ok((self.logical_range(view.point.position)?, Shape::Line))
            }
            Some(Operand::Motion(navigation)) => self.motion_range(navigation, size),
            Some(Operand::Object(object)) => self.object_range(object, request.amount()),
            Some(Operand::Resolved { origin, end }) if origin == self.view.point => Ok((
                origin.position.0.min(end.0)..origin.position.0.max(end.0),
                Shape::Linear,
            )),
            _ => Err(crate::coordinates::Error::Scalar(self.view.point.position).into()),
        }
    }
    fn motion_range(
        &self,
        request: Navigation,
        size: Size,
    ) -> Result<(Range<usize>, Shape), Error> {
        let (view, _) = self.navigation_view(request, size)?;
        let target = view.point.position;
        if matches!(
            request.target,
            Target::Line(_)
                | Target::Buffer(_)
                | Target::Page { .. }
                | Target::Screen(_)
                | Target::Paragraph(_)
        ) {
            return Ok((self.logical_range(target)?, Shape::Line));
        }
        let point = self.view.point.position;
        Ok((point.0.min(target.0)..point.0.max(target.0), Shape::Linear))
    }
    fn object_range(&self, object: Object, count: usize) -> Result<(Range<usize>, Shape), Error> {
        let range = Coordinates::new(&self.buffer)?
            .text_object(self.view.point.position, object, count)?
            .ok_or(crate::coordinates::Error::Scalar(self.view.point.position))?;
        let shape = if matches!(object, Object::Paragraph { .. }) {
            Shape::Line
        } else {
            Shape::Linear
        };
        Ok((range, shape))
    }
    fn logical_range(&self, target: Scalar) -> Result<Range<usize>, Error> {
        let coordinates = Coordinates::new(&self.buffer)?;
        let first = coordinates.to_line(self.view.point.position)?.row;
        let last = coordinates.to_line(target)?.row;
        let start = coordinates.from_cell(Cell {
            row: first.min(last),
            column: 0,
        })?;
        let last = first.max(last);
        let end = if last + 1 < self.buffer.len_lines() {
            coordinates.from_cell(Cell {
                row: last + 1,
                column: 0,
            })?
        } else {
            coordinates.end()
        };
        Ok(start.0..end.0)
    }
}

fn end_operand(request: Editing) -> Option<Operand> {
    (request.action == Action::CopyEnd).then(|| {
        let mut target = Navigation::new(Target::LineEnd);
        target.count = request.count;
        Operand::Motion(target)
    })
}

use super::{Action as HostAction, Key, Session};

impl Session {
    pub(super) fn edit_terminal_copy(&mut self, key: Key, request: Editing) -> HostAction {
        if !request.action.copies() {
            return self.error("Terminal copy text is read-only");
        }
        let copy = self
            .terminals
            .copies
            .get(&key)
            .expect("copy operator context");
        if copy.buffer.mark_active() {
            return self.copy_terminal_region(key);
        }
        if request.pending() {
            let outcome = self.input.expect_operator(request);
            return self.outcome(outcome, std::time::Instant::now());
        }
        match self.prepare_copy_operand(key, request) {
            Ok(entry) => {
                self.kills.publish_transfer(entry);
                self.message("Terminal target copied; copy viewport retained")
            }
            Err(error) => self.error(&error),
        }
    }
    fn prepare_copy_operand(
        &self,
        key: Key,
        request: Editing,
    ) -> Result<crate::kill::Entry, String> {
        let copy = self
            .terminals
            .copies
            .get(&key)
            .expect("copy operator context");
        let size = self
            .copy_size(key)
            .ok_or("Terminal copy presentation is suspended")?;
        let (span, shape) = copy.operator(request, size)?;
        self.kills
            .prepare_transfer(
                &copy.buffer,
                &span,
                crate::kill::Transfer {
                    view: self.windows.selected_id(),
                    backward: false,
                    shape,
                },
            )
            .map_err(|e| e.to_string())
    }
    pub(super) fn terminal_copy_rectangle(
        &mut self,
        key: Key,
        command: crate::command::RectangleCommand,
    ) -> HostAction {
        use crate::command::RectangleCommand;
        let copy = self
            .terminals
            .copies
            .get_mut(&key)
            .expect("copy rectangle context");
        if command == RectangleCommand::Mark {
            let result = copy
                .buffer
                .toggle_rectangle(copy.view.point)
                .map_err(|e| e.to_string());
            return self.copy_result(result);
        }
        if command != RectangleCommand::Copy {
            return self.error("Terminal copy text is read-only");
        }
        match self.capture_copy_rectangle(key) {
            Ok(rectangle) => {
                self.rectangle = Some(std::sync::Arc::new(rectangle));
                self.message("Terminal rectangle copied; copy viewport retained")
            }
            Err(error) => self.error(&error),
        }
    }
    fn capture_copy_rectangle(&self, key: Key) -> Result<crate::rectangle::Rectangle, String> {
        let copy = self
            .terminals
            .copies
            .get(&key)
            .expect("copy rectangle context");
        let selection = crate::rectangle::Selection::new(&copy.buffer, copy.view.point)
            .map_err(|e| e.to_string())?
            .ok_or("No terminal rectangle is selected")?;
        crate::rectangle::Rectangle::capture(
            &copy.buffer,
            &selection,
            crate::rectangle::Capture::Copy,
        )
        .map_err(|e| e.to_string())
    }
}
