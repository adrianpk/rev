use super::{Error, Session};
use crate::command::{Editing, editing::Action, navigation::Target};
use crate::coordinates::{Coordinates, Scalar};
use crate::search::Direction;

impl Session {
    pub(super) fn direct_edit(&mut self, request: Editing) -> Result<bool, Error> {
        match request.action {
            Action::Repeat => self.repeat_change(request.count),
            Action::DeleteForward => self.small_delete(request.amount(), false),
            Action::DeleteBackward => self.small_delete(request.amount(), true),
            Action::Replace => self.replace_character(request),
            Action::PasteAfter => self.paste_transfer(request.amount(), false),
            Action::PasteBefore => self.paste_transfer(request.amount(), true),
            Action::Join => self.join_lines(request.amount().max(2)),
            _ => Err(Error::Target),
        }
    }

    fn small_delete(&mut self, count: usize, backward: bool) -> Result<bool, Error> {
        let coordinates = Coordinates::new(self.buffer()?)?;
        let point = self.view().point.position;
        let direction = if backward {
            Direction::Backward
        } else {
            Direction::Forward
        };
        let target = coordinates.navigation(point, Target::Character(direction), count)?;
        let target = if backward {
            target.max(coordinates.line_start(point)?)
        } else {
            target.min(coordinates.line_end(point)?)
        };
        Ok(self.replace(point.0.min(target.0)..point.0.max(target.0), "")?)
    }

    fn replace_character(&mut self, request: Editing) -> Result<bool, Error> {
        let character = request.character.ok_or(Error::Target)?;
        let point = self.view().point.position;
        let Some(end) =
            Coordinates::new(self.buffer()?)?.replacement_end(point, request.amount())?
        else {
            return Err(Error::Target);
        };
        let text = super::transfer::repeat_text(&character.to_string(), request.amount())?;
        Ok(self.replace_cursor(point.0..end.0, &text, Scalar(point.0))?)
    }
}
