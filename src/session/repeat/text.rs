use super::{Pending, TEXT_BYTES, Trim};
use crate::buffer::{Stamp, edit::Error};
use crate::coordinates::{Coordinates, Scalar};
use crate::input::EditingState;
use crate::session::Session;
use unicode_segmentation::UnicodeSegmentation;

impl Session {
    pub(in crate::session) fn published_repeat_boundary(&mut self) {
        if !self.repeat.replaying {
            self.repeat.last = None;
        }
    }

    pub(in crate::session) fn repeat_group(&self) -> Option<Stamp> {
        self.repeat
            .text
            .then_some(self.repeat.pending.as_ref()?.first)
            .flatten()
    }

    pub(in crate::session) fn capture_repeat_text(&mut self, joined: bool) {
        if !self.repeat.text || self.repeat.replaying {
            return;
        }
        let Some(mut pending) = self.repeat.pending.take() else {
            return;
        };
        if pending.first.is_some() && !joined {
            pending.valid = false;
        }
        self.repeat.last = None;
        let transaction = self
            .buffer()
            .expect("published text owner")
            .history()
            .back()
            .expect("published edit");
        pending.first = Some(transaction.before);
        pending.point = self.view().point;
        pending.changed = true;
        let Some(change) = transaction.changes().first() else {
            return;
        };
        retain_text(
            &mut pending,
            change,
            self.buffer().expect("published source").newline(),
        );
        self.repeat.pending = Some(pending);
    }

    pub(in crate::session) fn entry_text(&mut self, text: &str) -> Result<bool, Error> {
        if !self.editing_state().is_some_and(EditingState::text_entry) {
            let at = self.view().point.position.0;
            return self.edit(at..at, text, true, None);
        }
        self.begin_repeat_text();
        let range = self.entry_text_range(text)?;
        self.repeat.text = true;
        let result = self.edit(range, text, false, None);
        self.repeat.text = false;
        result
    }

    pub(in crate::session) fn entry_text_range(
        &self,
        text: &str,
    ) -> Result<std::ops::Range<usize>, Error> {
        let point = self.view().point.position;
        if self.editing_state() != Some(EditingState::Replace) {
            return Ok(point.0..point.0);
        }
        self.overwrite_range(point, text)
    }

    pub(in crate::session) fn overwrite_range(
        &self,
        point: Scalar,
        text: &str,
    ) -> Result<std::ops::Range<usize>, Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let count = text
            .graphemes(true)
            .filter(|g| !g.contains(['\r', '\n']))
            .count();
        let count = count.saturating_sub(joined_graphemes(self, point, text)?);
        let end = coordinates.navigation(
            point,
            crate::command::navigation::Target::Character(crate::search::Direction::Forward),
            count.max(1),
        )?;
        let end = if count == 0 {
            point
        } else {
            end.min(coordinates.line_end(point)?)
        };
        Ok(point.0..end.0)
    }
}

fn joined_graphemes(session: &Session, point: Scalar, text: &str) -> Result<usize, Error> {
    if text.is_empty() || point.0 == 0 {
        return Ok(0);
    }
    let buffer = session.buffer()?;
    let coordinates = Coordinates::new(buffer)?;
    if point == coordinates.line_start(point)? {
        return Ok(0);
    }
    let start = coordinates.previous(point)?;
    let previous = buffer.slice(&buffer.span(start.0..point.0)?)?.to_string();
    let combined = format!("{previous}{text}");
    Ok(
        (previous.graphemes(true).count() + text.graphemes(true).count())
            .saturating_sub(combined.graphemes(true).count()),
    )
}

fn retain_text(pending: &mut Pending, change: &crate::buffer::edit::Change, newline: &str) {
    if change.inserted().len_bytes() > TEXT_BYTES + 2 {
        pending.valid = false;
    }
    if pending.valid {
        let text = change.inserted().to_string();
        let text = match pending.trim {
            Trim::Prefix => text.strip_prefix(newline).unwrap_or(&text),
            Trim::Suffix => text.strip_suffix(newline).unwrap_or(&text),
            Trim::None => &text,
        };
        pending.valid = text.len() <= TEXT_BYTES;
        if pending.valid {
            pending.record.text = text.to_owned();
        }
    }
    if !pending.valid {
        pending.record.text.clear();
    }
}
