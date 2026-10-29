//! Qualified observations for explicit source actions, never cached edit plans.
use super::{CalendarSource, SourceVersion};
use crate::{
    buffer::{Buffers, Stamp, edit::Point},
    coordinates::Scalar,
};
#[derive(Clone, Debug)]
pub struct ActionTarget {
    pub source: CalendarSource,
    pub byte: usize,
    pub title: std::sync::Arc<str>,
}
impl ActionTarget {
    pub fn same(&self, other: &Self) -> bool {
        self.byte == other.byte
            && self.source.path == other.source.path
            && self.source.version == other.source.version
    }
    pub fn point(&self, buffers: &Buffers) -> Result<Point, String> {
        let SourceVersion::Buffer(stamp) = self.source.version else {
            return Err("Disk agenda target requires a checked source read".into());
        };
        let buffer = buffers.get(stamp.buffer).map_err(|e| e.to_string())?;
        if buffer.stamp() != stamp {
            return Err("Agenda source revision changed; refresh and select again".into());
        }
        let position = buffer
            .byte_span(self.byte..self.byte)
            .map_err(|e| e.to_string())?
            .range
            .start;
        Ok(Point {
            stamp,
            position: Scalar(position),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionStatus {
    Changed,
    Unchanged,
    Rejected,
    Cancelled,
    ChangedWithFailure,
}
#[derive(Clone, Debug)]
pub struct ActionResult {
    pub target: ActionTarget,
    pub status: ActionStatus,
    pub message: String,
    pub after: Option<Stamp>,
}

impl ActionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Changed => "Changed",
            Self::Unchanged => "Unchanged",
            Self::Rejected => "Rejected",
            Self::Cancelled => "Cancelled",
            Self::ChangedWithFailure => "Changed; completion unresolved",
        }
    }
}
