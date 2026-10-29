use super::*;
use crate::buffer::{Buffers, Visit};
pub(crate) struct Navigation {
    pub(super) destination: crate::org::refile::Destination,
    pub(super) byte: usize,
}
impl Navigation {
    /// Recheck the actual registry owner, immutable bytes and file identity
    /// before qualifying a point or publishing a newly visited destination.
    pub(crate) fn prepare(&self, buffers: &Buffers) -> Result<(Visit, Point), String> {
        let visit = self.destination.visit(buffers)?;
        let buffer = visit.buffer(buffers).map_err(|e| e.to_string())?;
        self.destination.check(buffer)?;
        if let Some(file) = buffer.file() {
            let current = crate::file::resolve(file.path()).map_err(|e| e.to_string())?;
            if current.observation() != file.observation() {
                return Err("Link destination changed externally".into());
            }
        }
        let position = crate::coordinates::Scalar(
            buffer
                .byte_span(self.byte..self.byte)
                .map_err(|e| e.to_string())?
                .range
                .start,
        );
        let point = Point {
            stamp: buffer.stamp(),
            position,
        };
        Ok((visit, point))
    }
}
