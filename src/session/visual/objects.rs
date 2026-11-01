use crate::buffer::Visual;
use crate::command::{
    Editing,
    editing::{Object, Operand},
};
use crate::coordinates::{Coordinates, Scalar};
use crate::input::EditingState;
use crate::session::{Action, Session, editing::Error};

impl Session {
    pub(super) fn visual_object(&mut self, request: Editing) -> Action {
        if !matches!(
            self.editing_state(),
            Some(EditingState::VisualCharacter | EditingState::VisualLine)
        ) {
            return self.message("Text objects require a linear Visual selection");
        }
        if request.operand.is_none() {
            let outcome = self.input.expect_visual_object(request);
            return self.outcome(outcome, std::time::Instant::now());
        }
        match self.select_visual_object(request) {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn select_visual_object(&mut self, request: Editing) -> Result<(), Error> {
        let Some(Operand::Object(object)) = request.operand else {
            return Err(Error::Target);
        };
        if !matches!(
            object,
            Object::Word { big: false, .. } | Object::Paragraph { .. }
        ) {
            return Err(Error::Target);
        }
        let coordinates = Coordinates::new(self.buffer()?)?;
        let range = coordinates
            .text_object(self.view().point.position, object, request.amount())?
            .filter(|range| !range.is_empty())
            .ok_or(Error::Target)?;
        let end = coordinates.previous_observed(Scalar(range.end))?;
        self.publish_visual_object(Scalar(range.start), end)
    }

    fn publish_visual_object(&mut self, start: Scalar, end: Scalar) -> Result<(), Error> {
        let mut point = self.view().point;
        point.position = end;
        let mut candidate = self.view().clone();
        candidate.point = point;
        candidate.reveal(self.buffer()?, self.pane_size())?;
        let buffer = self.buffers.get_mut(point.stamp.buffer)?;
        buffer.activate_mark(crate::buffer::edit::Point {
            position: start,
            ..point
        })?;
        buffer.activate_visual(
            crate::buffer::edit::Point {
                position: start,
                ..point
            },
            Visual::Character,
        )?;
        *self.windows.selected_mut() = candidate;
        self.input.set_editing_state(EditingState::VisualCharacter);
        Ok(())
    }
}
