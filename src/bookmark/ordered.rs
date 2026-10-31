use super::{Bookmarks, RecordId, Resolution};
use crate::buffer::Stamp;
use std::path::Path;

pub(crate) struct Location<'a> {
    pub id: RecordId,
    pub position: usize,
    pub name: &'a str,
}
impl Bookmarks {
    /// Current-file traversal considers only revision-current resolved anchors;
    /// it never opens a source to guess the order of durable unresolved records.
    pub(crate) fn ordered(&self, stamp: Stamp, path: &Path) -> Vec<Location<'_>> {
        let mut locations: Vec<_> = self
            .iter()
            .filter(|r| {
                r.path() == path
                    && r.point().stamp == stamp
                    && r.resolution() == Resolution::Resolved
            })
            .map(|r| Location {
                id: r.id(),
                position: r.point().position.0,
                name: r.name(),
            })
            .collect();
        locations.sort_by(|a, b| a.position.cmp(&b.position).then_with(|| a.name.cmp(b.name)));
        locations
    }
}
