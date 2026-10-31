use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use super::Resolution;
use crate::buffer::{Buffer, Stamp};
use crate::coordinates::Coordinates;

/// One buffer-owned derived index. Its lifetime cannot outlive the source, and
/// its key also rejects record mutations and file-identity changes without edits.
#[derive(Default)]
pub(crate) struct Cache(Option<Index>);

struct Index {
    stamp: Stamp,
    generation: u64,
    path: PathBuf,
    rows: Arc<BTreeSet<usize>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bookmark::Context;
    use crate::buffer::{Buffers, edit::Point};
    use crate::coordinates::Scalar;
    use crate::fixture::Fixture;

    // Visible-row queries and peer panels reuse one index. Record-only changes
    // reject it without relying on a source edit to advance the buffer stamp.
    #[test]
    fn bookmark_index() {
        let f = Fixture::new();
        let path = f.write("source", "first\nsecond\n");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&path).unwrap();
        let buffer = buffers.get(id).unwrap();
        let set = |position| {
            buffers
                .bookmarks_mut()
                .set(
                    "A",
                    path.clone(),
                    Point {
                        stamp: buffer.stamp(),
                        position: Scalar(position),
                    },
                    true,
                    Context::capture(buffer, Scalar(position)).unwrap(),
                )
                .unwrap();
        };
        set(6);
        let original = buffer.bookmark_rows().unwrap();
        assert_eq!(original.iter().copied().collect::<Vec<_>>(), [1]);
        assert!(Arc::ptr_eq(&original, &buffer.bookmark_rows().unwrap()));
        set(0);
        let changed = buffer.bookmark_rows().unwrap();
        assert!(!Arc::ptr_eq(&original, &changed));
        assert_eq!(changed.iter().copied().collect::<Vec<_>>(), [0]);
        assert!(Arc::ptr_eq(&changed, &buffer.bookmark_rows().unwrap()));
        buffers.bookmarks_mut().delete("A").unwrap();
        assert!(buffer.bookmark_rows().unwrap().is_empty());
        assert_eq!(original.iter().copied().collect::<Vec<_>>(), [1]);
    }
}

impl Buffer {
    /// Obtain a revision-current set once per panel, then query visible logical
    /// rows. Only resolved live anchors contribute; this never reads a file,
    /// relocates a stored target or changes the canonical collection.
    pub(crate) fn bookmark_rows(&self) -> Result<Arc<BTreeSet<usize>>, crate::view::Error> {
        let Some(file) = self.file() else {
            return Ok(Arc::default());
        };
        let records = self.bookmarks.lock().expect("bookmark collection poisoned");
        let mut cache = self.bookmark_rows.lock().expect("bookmark index poisoned");
        if let Some(index) = &cache.0
            && index.stamp == self.stamp()
            && index.generation == records.generation()
            && index.path == file.path()
        {
            return Ok(index.rows.clone());
        }
        let coordinates = Coordinates::new(self)?;
        let rows = records
            .iter()
            .filter(|record| {
                record.resolution() == Resolution::Resolved
                    && record.point().stamp == self.stamp()
                    && record.path() == file.path()
            })
            .map(|record| {
                coordinates
                    .to_line(record.point().position)
                    .map(|line| line.row)
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        let rows = Arc::new(rows);
        cache.0 = Some(Index {
            stamp: self.stamp(),
            generation: records.generation(),
            path: file.path().to_path_buf(),
            rows: rows.clone(),
        });
        Ok(rows)
    }
}
