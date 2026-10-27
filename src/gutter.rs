//! Native view policy and disposable number geometry. These records prepare
//! numbering without enabling rendering, TOML keys or command identities.

mod geometry;
mod policy;

pub use geometry::{Error, Geometry};
pub use policy::{Change, CurrentLine, Format, Mode, Overrides, Policy, Settings, Side};

use crate::buffer::{Buffer, Span, Stamp};
use crate::coordinates::Coordinates;
use crate::view::{self, View};
use crate::window::{Area, ViewId};

/// One live editing panel's source and layout snapshot. It owns no source text
/// or per-line labels and must be derived again after source/view/layout changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projection {
    pub view: ViewId,
    pub source: Stamp,
    pub lines: usize,
    /// Zero-based logical point row, independent of scroll or terminal columns.
    pub current: usize,
    pub policy: Settings,
    pub geometry: Geometry,
}

impl Projection {
    pub(crate) fn new(
        id: ViewId,
        view: &View,
        buffer: &Buffer,
        content: Area,
    ) -> Result<Self, view::Error> {
        let position = view.point.position;
        buffer.slice(&Span {
            stamp: view.point.stamp,
            range: position.0..position.0,
        })?;
        let current = Coordinates::new(buffer)?.to_line(position)?.row;
        let policy = view.gutter.effective();
        Ok(Self {
            view: id,
            source: buffer.stamp(),
            lines: buffer.len_lines(),
            current,
            policy,
            geometry: Geometry::new(content, buffer.len_lines(), policy)?,
        })
    }
}
