use ropey::Rope;

use crate::buffer::{Buffer, Span, Stamp, edit::Error};
use crate::coordinates::Scalar;
use crate::window::ViewId;

#[derive(Clone, Copy)]
struct Continuation {
    stamp: Stamp,
    view: ViewId,
    point: Scalar,
}

/// One retained kill entry, coalesced only through a checked live command chain.
#[derive(Default)]
pub(crate) struct KillText {
    text: Option<Rope>,
    continuation: Option<Continuation>,
}

impl KillText {
    pub(crate) fn text(&self) -> Option<&Rope> {
        self.text.as_ref()
    }

    pub(crate) fn break_chain(&mut self) {
        self.continuation = None;
    }

    /// Prepare owned fragments before text publication; a rejected edit drops
    /// this candidate without replacing the previously retained kill text.
    pub(crate) fn prepare(
        &self,
        buffer: &Buffer,
        view: ViewId,
        source: &Span,
        backward: bool,
    ) -> Result<Rope, Error> {
        let mut fragment: Rope = buffer.slice(source)?.into();
        let point = Scalar(if backward {
            source.range.end
        } else {
            source.range.start
        });
        let Some(previous) = self
            .text
            .as_ref()
            .filter(|_| self.continues(source.stamp, view, point))
        else {
            return Ok(fragment);
        };
        if backward {
            fragment.append(previous.clone());
            Ok(fragment)
        } else {
            let mut text = previous.clone();
            text.append(fragment);
            Ok(text)
        }
    }

    fn continues(&self, stamp: Stamp, view: ViewId, point: Scalar) -> bool {
        self.continuation.is_some_and(|previous| {
            previous.stamp == stamp && previous.view == view && previous.point == point
        })
    }

    pub(crate) fn publish(&mut self, text: Rope, stamp: Stamp, view: ViewId, point: Scalar) {
        self.text = Some(text);
        self.continuation = Some(Continuation { stamp, view, point });
    }
}
