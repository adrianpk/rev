use crate::buffer::{
    Span, Whitespace,
    edit::{Cursor, Error, Replacement},
};
use crate::coordinates::Scalar;
use crate::input::Focus;

use super::{Action, Session, linear};

impl Session {
    pub(super) fn whitespace_command(&mut self, operation: Whitespace) -> Action {
        if self.input.focus() != Focus::Editing {
            return self.message("a prompt owns input focus");
        }
        match self.convert_whitespace(operation) {
            Ok(changed) => self.finish(Ok(changed)),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn convert_whitespace(&mut self, operation: Whitespace) -> Result<bool, linear::Error> {
        let source = self.active_region()?;
        if source.range.is_empty() {
            return Ok(false);
        }
        let buffer = self
            .buffers
            .get_mut(source.stamp.buffer)
            .map_err(Error::from)?;
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.into());
        }
        let edits = buffer.whitespace(&source, operation)?;
        if edits.is_empty() {
            return Ok(false);
        }
        Ok(self.publish_whitespace(&source, edits)?)
    }

    fn publish_whitespace(
        &mut self,
        source: &Span,
        edits: Vec<Replacement>,
    ) -> Result<bool, Error> {
        let buffer = self.buffers.get_mut(source.stamp.buffer)?;
        let points = self.windows.points(buffer)?;
        let cursor = Cursor {
            index: points.initiator,
            position: endpoint(&edits, &source, points.values[points.initiator].position)?,
        };
        let mut prepared = buffer.prepare(&edits, &points.values, Some(cursor))?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.windows.selected_mut().reset_navigation();
        Ok(true)
    }
}

// The invoking point is one region endpoint. Other views keep normal mapping;
// the initiating start/end is retained explicitly for coherent history replay.
fn endpoint(edits: &[Replacement], source: &Span, point: Scalar) -> Result<Scalar, Error> {
    if point.0 == source.range.start {
        return Ok(point);
    }
    edits.iter().try_fold(point, |point, edit| {
        point
            .0
            .checked_sub(edit.source.range.len())
            .and_then(|position| position.checked_add(edit.text.chars().count()))
            .map(Scalar)
            .ok_or(Error::Capacity)
    })
}
