use super::{Intent, Payload, Record, selection::Geometry};
use crate::command::{
    Editing, ModalCommand,
    editing::{Action, Operand},
};
use crate::coordinates::Scalar;
use crate::input::{EditingState, Focus};
use crate::kill::Shape;
use crate::session::{Session, editing::Error};

impl Session {
    pub(in crate::session) fn repeat_change(&mut self, count: Option<u32>) -> Result<bool, Error> {
        if self.editing_state() != Some(EditingState::Normal)
            || self.input.focus() != Focus::Editing
            || self.tree.focused()
        {
            return Err(Error::Target);
        }
        self.view().check_point(self.buffer()?)?;
        if self.buffer()?.is_readonly() {
            return Err(crate::buffer::edit::Error::ReadOnly.into());
        }
        let record = self.repeat.last.clone().ok_or(Error::Target)?;
        self.repeat.replaying = true;
        let result = self.apply_repeat(&record, count);
        self.repeat.replaying = false;
        result
    }

    fn apply_repeat(&mut self, record: &Record, count: Option<u32>) -> Result<bool, Error> {
        match &record.intent {
            Intent::Entry(entry) => {
                self.repeat_entry(*entry, &record.text, count.unwrap_or(1).max(1) as usize)
            }
            Intent::Edit(original, geometry) => {
                let request = override_count(*original, count);
                self.repeat_edit(request, *geometry, &record.text)
            }
            Intent::Paste { request, payload } => {
                self.repeat_paste(payload, override_count(*request, count))
            }
        }
    }

    fn repeat_entry(
        &mut self,
        entry: ModalCommand,
        text: &str,
        count: usize,
    ) -> Result<bool, Error> {
        let at = self.modal_target(entry)?;
        let mut insertion = crate::session::editing::transfer::repeat_text(text, count)?;
        let mut range = at.0..at.0;
        let cursor;
        match entry {
            ModalCommand::OpenAbove => {
                cursor = Scalar(at.0 + insertion.chars().count());
                insertion.push_str(self.buffer()?.newline());
            }
            ModalCommand::OpenBelow => {
                insertion.insert_str(0, self.buffer()?.newline());
                cursor = Scalar(at.0 + insertion.chars().count());
            }
            ModalCommand::Replace => {
                range = self.overwrite_range(at, &insertion)?;
                cursor = Scalar(at.0 + insertion.chars().count());
            }
            _ => cursor = Scalar(at.0 + insertion.chars().count()),
        }
        Ok(self.replace_cursor(range, &insertion, cursor)?)
    }

    fn repeat_edit(
        &mut self,
        request: Editing,
        geometry: Option<Geometry>,
        text: &str,
    ) -> Result<bool, Error> {
        if let Some(geometry @ Geometry::Block { .. }) = geometry {
            return self.repeat_block(request, geometry, text);
        }
        if request.action.changes() {
            let selection = match geometry {
                Some(geometry) => self.repeat_selection(geometry, request.count)?,
                None => self.editing_range(request)?.ok_or(Error::Target)?,
            };
            return self.repeat_change_selection(request, selection, text);
        }
        if let Some(geometry) = geometry {
            let selection = self.repeat_selection(geometry, request.count)?;
            return self.apply_selection(request, selection);
        }
        self.apply_editing(request)
    }

    fn repeat_change_selection(
        &mut self,
        request: Editing,
        selection: crate::session::editing::Selection,
        text: &str,
    ) -> Result<bool, Error> {
        let transfer = crate::kill::Transfer {
            view: self.windows.selected_id(),
            backward: selection.backward,
            shape: selection.shape,
        };
        let entry = self
            .kills
            .prepare_transfer(self.buffer()?, &selection.source, transfer)?;
        let mut insertion = text.to_owned();
        insertion.push_str(&self.change_line_text(request, &selection)?);
        let cursor = Scalar(selection.source.range.start + text.chars().count());
        let changed = self.replace_cursor(selection.source.range, &insertion, cursor)?;
        if changed {
            self.kills.publish_transfer(entry);
        }
        Ok(changed)
    }

    fn repeat_paste(&mut self, payload: &Payload, request: Editing) -> Result<bool, Error> {
        let before = request.action == Action::PasteBefore;
        match payload {
            Payload::Linear(Shape::Line, text) => self.paste_line(text, request.amount(), before),
            Payload::Linear(_, text) => self.paste_linear(text, request.amount(), before),
            Payload::Block(block) => self.paste_block(block, request.amount(), before),
        }
    }
}

pub(super) fn override_count(mut request: Editing, count: Option<u32>) -> Editing {
    if let Some(count) = count {
        request.count = Some(count.max(1));
        if let Some(Operand::Motion(mut motion)) = request.operand {
            motion.count = request.count;
            request.operand = Some(Operand::Motion(motion));
        }
    }
    request
}
