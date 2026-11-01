use crate::buffer::Visual;
use crate::command::{Editing, editing::Action as EditAction};
use crate::input::EditingState;
use crate::kill::Shape;
use crate::rectangle::{Capture, Rectangle, Selection as Block};
use crate::session::{
    Action, Session,
    editing::{Error, Selection},
};
use std::sync::Arc;

impl Session {
    pub(in crate::session) fn visual_editing(&mut self, request: Editing) -> Option<Action> {
        if (self.visual_source.is_some() || request.action.selects())
            && (self.input.focus() != crate::input::Focus::Editing || self.tree.focused())
        {
            return Some(self.message("A component owns input focus"));
        }
        self.visual_request(request)
    }

    fn visual_request(&mut self, request: Editing) -> Option<Action> {
        if request.action.selects() {
            return Some(self.visual_object(request));
        }
        let shape = self.editing_state()?.visual()?;
        if !request.action.operator() {
            return None;
        }
        let result = self.apply_visual(request, shape);
        Some(match result {
            Ok(changed) => {
                self.leave_visual();
                self.input.set_editing_state(if request.action.changes() {
                    EditingState::Insert
                } else {
                    EditingState::Normal
                });
                self.finish(Ok(changed))
            }
            Err(error) => self.error(&error.to_string()),
        })
    }

    fn apply_visual(&mut self, request: Editing, shape: Visual) -> Result<bool, Error> {
        let buffer = self.buffer()?;
        if self.visual_source != Some(buffer.stamp()) || buffer.visual() != Some(shape) {
            return Err(Error::Target);
        }
        if !request.action.copies() && buffer.is_readonly() {
            return Err(crate::buffer::edit::Error::ReadOnly.into());
        }
        if shape == Visual::Block {
            return self.edit_visual_block(request);
        }
        let source = buffer.region(self.view().point)?.ok_or(Error::Target)?;
        let selection = Selection {
            source,
            shape: if shape == Visual::Line {
                Shape::Line
            } else {
                Shape::Linear
            },
            backward: buffer
                .mark()
                .is_some_and(|mark| self.view().point.position < mark),
        };
        self.apply_selection(request, selection)
    }

    fn remove_visual_block(&mut self, selection: &Block) -> Result<bool, Error> {
        let edits = selection.deletions(self.buffer()?)?;
        let cursor = selection.cursor(self.buffer()?)?;
        Ok(self.replace_many(&edits, Some(cursor))?)
    }

    fn edit_visual_block(&mut self, request: Editing) -> Result<bool, Error> {
        if request.action.shifts() {
            return Err(Error::Target);
        }
        let buffer = self.buffer()?;
        let selection = Block::new(buffer, self.view().point)?.ok_or(Error::Target)?;
        let capture = if request.action == EditAction::Copy {
            Capture::Copy
        } else {
            Capture::Kill
        };
        let rectangle = Arc::new(Rectangle::capture(buffer, &selection, capture)?);
        let entry = self.kills.prepare_block(rectangle.clone());
        let changed = if request.action.copies() {
            false
        } else {
            self.remove_visual_block(&selection)?
        };
        self.kills.publish_transfer(entry);
        self.rectangle = Some(rectangle);
        self.yank = None;
        Ok(changed)
    }
}
