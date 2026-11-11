use std::ops::Range;

use crate::buffer::{Span, edit};
use crate::window::ViewId;

use super::{Session, linear::Error};

pub(super) struct Capture {
    source: Span,
    view: ViewId,
    rotation: usize,
}

impl Session {
    pub(super) fn yank(&mut self) -> Result<bool, Error> {
        let text = self.kills.text().ok_or(Error::EmptyRing)?;
        self.buffer()
            .map_err(edit::Error::from)?
            .check_edit_payload(text.len_bytes())?;
        let text = text.to_string();
        let point = self.windows.selected().point.position.0;
        Ok(self.publish_yank(point..point, &text, 0)?)
    }

    pub(super) fn yank_pop(&mut self) -> Result<bool, Error> {
        let capture = self.yank.take().ok_or(Error::Yank)?;
        if capture.view != self.windows.selected_id()
            || capture.source.stamp != self.windows.selected().point.stamp
        {
            return Err(Error::Yank);
        }
        let (rotation, text) = self.kills.older(capture.rotation).ok_or(Error::Yank)?;
        self.buffer()
            .map_err(edit::Error::from)?
            .check_edit_payload(text.len_bytes())?;
        let text = text.to_string();
        Ok(self.publish_yank(capture.source.range, &text, rotation)?)
    }

    fn publish_yank(
        &mut self,
        range: Range<usize>,
        text: &str,
        rotation: usize,
    ) -> Result<bool, edit::Error> {
        self.publish_yank_at(range, text, rotation, None)
    }

    pub(super) fn publish_yank_at(
        &mut self,
        range: Range<usize>,
        text: &str,
        rotation: usize,
        cursor: Option<crate::coordinates::Scalar>,
    ) -> Result<bool, edit::Error> {
        let end = range
            .start
            .checked_add(text.chars().count())
            .ok_or(edit::Error::Capacity)?;
        // Capture the literal inserted span, not the grapheme-adjusted cursor:
        // an insertion can join a cluster or CRLF with untouched neighboring text.
        let inserted = range.start..end;
        let changed = if let Some(cursor) = cursor {
            self.replace_cursor(range, text, cursor)?
        } else {
            self.replace(range, text)?
        };
        self.yank = Some(Capture {
            source: Span {
                stamp: self.windows.selected().point.stamp,
                range: inserted,
            },
            view: self.windows.selected_id(),
            rotation,
        });
        Ok(changed)
    }
}

impl Session {
    pub(super) fn argument_yank(&mut self, invocation: crate::input::Invocation) -> super::Action {
        use crate::command::{ArgumentForm, LinearCommand};
        if self.argument_yank_read_only() {
            return self.error("Source text is read-only");
        }
        let value = invocation.argument.value();
        let plain = matches!(invocation.argument.form(), ArgumentForm::PlainPrefix { .. });
        let pop = invocation.command == crate::command::Command::Linear(LinearCommand::Pop);
        if pop && value == 0 {
            return self.message("");
        }
        if !pop && !plain && value <= 0 {
            return self.error("Yank requires a positive entry number");
        }
        self.command_boundary(invocation.command);
        self.finish_repeat_entry();
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        let result = self.argument_yank_entry(value, plain, pop);
        self.sync_input_context();
        match result {
            Ok(changed) => self.finish(Ok(changed)),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn argument_yank_read_only(&self) -> bool {
        self.context_storage.locked() || self.terminals.focused || self.terminal_copy_active()
    }

    fn argument_yank_entry(&mut self, value: i32, plain: bool, pop: bool) -> Result<bool, Error> {
        if pop {
            self.rotate_argument_yank(value)
        } else {
            let index = if plain {
                0
            } else {
                usize::try_from(value - 1).expect("positive yank index")
            };
            self.select_argument_yank(index, plain)
        }
    }

    fn rotate_argument_yank(&mut self, distance: i32) -> Result<bool, Error> {
        let capture = self.yank.take().ok_or(Error::Yank)?;
        if capture.view != self.windows.selected_id()
            || capture.source.stamp != self.view().point.stamp
        {
            return Err(Error::Yank);
        }
        let (rotation, text) = self
            .kills
            .rotate(capture.rotation, distance)
            .ok_or(Error::Yank)?;
        self.buffer()
            .map_err(edit::Error::from)?
            .check_edit_payload(text.len_bytes())?;
        let text = text.to_string();
        Ok(self.publish_yank(capture.source.range, &text, rotation)?)
    }

    fn select_argument_yank(&mut self, index: usize, exchange: bool) -> Result<bool, Error> {
        let text = self.kills.entry(index).ok_or(Error::EmptyRing)?;
        self.buffer()
            .map_err(edit::Error::from)?
            .check_edit_payload(text.len_bytes())?;
        let text = text.to_string();
        let at = self.view().point.position.0;
        if !exchange {
            return Ok(self.publish_yank(at..at, &text, index)?);
        }
        self.exchange_argument_yank(text, index, at)
    }

    fn exchange_argument_yank(
        &mut self,
        text: String,
        index: usize,
        at: usize,
    ) -> Result<bool, Error> {
        // Prepare point and mark with the source transaction. No fallible
        // endpoint change remains after publication, including joined clusters.
        let buffer = self
            .buffers
            .get_mut(self.view().point.stamp.buffer)
            .map_err(edit::Error::from)?;
        let points = self.windows.points(buffer)?;
        let end = at
            .checked_add(text.chars().count())
            .ok_or(edit::Error::Capacity)?;
        let replacement = edit::Replacement {
            source: buffer.span(at..at).map_err(edit::Error::from)?,
            text,
        };
        let mut prepared = buffer.prepare(
            &[replacement],
            &points.values,
            Some(edit::Cursor {
                index: points.initiator,
                position: crate::coordinates::Scalar(at),
            }),
        )?;
        prepared.set_mark(crate::coordinates::Scalar(end))?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.published_repeat_boundary();
        self.windows.selected_mut().reset_navigation();
        self.yank = Some(Capture {
            source: Span {
                stamp: self.view().point.stamp,
                range: at..end,
            },
            view: self.windows.selected_id(),
            rotation: index,
        });
        self.sync_input_context();
        Ok(true)
    }
}
