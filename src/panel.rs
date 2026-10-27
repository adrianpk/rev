//! Derived presentation bounds. Panels own no buffers, views or input state.
//! Shared separators belong to composition, outside each panel's outer bounds.

mod border;
mod composition;

pub use composition::Composition;

use std::ops::Range;

use crate::display::Position;
use crate::window::Area;

/// One clipped presentation surface with an optional final status row.
/// Activity is presentation metadata supplied by the semantic owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Panel {
    outer: Area,
    content: Area,
    status: Option<Area>,
    active: bool,
}

impl Panel {
    pub fn new(outer: Area, status: bool, active: bool) -> Self {
        let (content, status) = outer.with_status(status);
        Self {
            outer,
            content,
            status,
            active,
        }
    }

    pub fn outer(self) -> Area {
        self.outer
    }
    pub fn content(self) -> Area {
        self.content
    }
    pub fn status(self) -> Option<Area> {
        self.status
    }
    pub fn active(self) -> bool {
        self.active
    }

    /// Translate a content-local cell once. Status, separators and cells beyond
    /// the visible content cannot become document cursors.
    pub fn position(self, local: Position) -> Option<Position> {
        if local.column >= self.content.width() || local.row >= self.content.height() {
            return None;
        }
        Some(Position {
            column: self.content.column() + local.column,
            row: self.content.row() + local.row,
        })
    }

    /// Clip a content-local cell span and return its surface row/columns.
    /// Callers retain responsibility for whole-grapheme text clipping.
    pub fn clip(self, row: u16, columns: Range<u16>) -> Option<(u16, Range<u16>)> {
        let end = columns.end.min(self.content.width());
        if columns.start >= end {
            return None;
        }
        let start = self.position(Position {
            column: columns.start,
            row,
        })?;
        Some((start.row, start.column..self.content.column() + end))
    }
}
