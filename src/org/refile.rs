//! Revision-qualified complete subtree moves. Observations and plans have no
//! publication authority; the canonical edit owner admits all participants.
mod destination;
mod plan;
mod targets;
#[cfg(test)]
mod tests;
use crate::{
    buffer::{Buffer, Span, Stamp, edit::Point},
    org::{Cancellation, ColorScan, Limits},
};
pub(in crate::org) use destination::Destination;
pub use plan::Plan;
use ropey::Rope;
pub use targets::{Target, Targets};

#[derive(Clone)]
pub struct Subtree {
    source: Span,
    text: Rope,
}
impl Subtree {
    /// Observe and qualify complete sibling subtrees at an actual source point.
    /// Interactive callers reuse their already admitted structural observation.
    pub fn at(buffer: &Buffer, point: Point, cancel: &Cancellation) -> Result<Self, String> {
        if point.stamp != buffer.stamp() || !buffer.is_org() {
            return Err("Refile requires a current native Org source point".into());
        }
        let coloring = ColorScan::new(buffer, Limits::default())
            .map_err(|e| e.to_string())?
            .run(cancel);
        let (_, _, outline) = coloring.into_parts();
        let span = outline.refile_span(buffer, point.position)?;
        Self::checked(buffer, span)
    }
    pub(crate) fn checked(buffer: &Buffer, source: Span) -> Result<Self, String> {
        let text = buffer.slice(&source).map_err(|e| e.to_string())?.into();
        Ok(Self { source, text })
    }
    pub fn stamp(&self) -> Stamp {
        self.source.stamp
    }
    pub fn source(&self) -> &Span {
        &self.source
    }
    pub(in crate::org) fn text(&self) -> &Rope {
        &self.text
    }
    pub(in crate::org) fn current(&self, buffer: &Buffer) -> Result<(), String> {
        if buffer.slice(&self.source).map_err(|e| e.to_string())? != self.text.slice(..) {
            return Err("Refile source changed; nothing moved".into());
        }
        Ok(())
    }
}
