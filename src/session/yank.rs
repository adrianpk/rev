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
        let text = self.kills.text().ok_or(Error::EmptyRing)?.to_string();
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
        let text = text.to_string();
        Ok(self.publish_yank(capture.source.range, &text, rotation)?)
    }

    fn publish_yank(
        &mut self,
        range: Range<usize>,
        text: &str,
        rotation: usize,
    ) -> Result<bool, edit::Error> {
        let end = range
            .start
            .checked_add(text.chars().count())
            .ok_or(edit::Error::Capacity)?;
        // Capture the literal inserted span, not the grapheme-adjusted cursor:
        // an insertion can join a cluster or CRLF with untouched neighboring text.
        let inserted = range.start..end;
        let changed = self.replace(range, text)?;
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
