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
