mod build;
mod role;

pub use role::Role;

use std::ops::Range;

use crate::buffer::{Buffer, Error, Stamp};
use crate::org::{Cancellation, Coverage, Index, Limits, Scan, Source, StructureScan};
use crate::syntax::Capture;

/// Read-only presentation offsets. Every query rejects a different canonical
/// buffer/revision before returning any style, including a retained old cache.
pub struct Highlights {
    stamp: Stamp,
    coverage: Coverage,
    captures: Vec<Capture>,
    ends: Vec<usize>,
    reindexed: usize,
}

impl Highlights {
    pub(crate) fn check(&self, buffer: &Buffer) -> Result<(), Error> {
        super::source::current(buffer, self.stamp)
    }

    pub fn stamp(&self) -> Stamp {
        self.stamp
    }
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }
    /// Index suffix work only; structural/inline ownership is re-observed from
    /// the shared snapshot so delimiter recovery cannot inherit stale context.
    pub fn reindexed_bytes(&self) -> usize {
        self.reindexed
    }

    pub fn captures(&self, buffer: &Buffer, bytes: Range<usize>) -> Vec<Capture> {
        if super::source::current(buffer, self.stamp).is_err()
            || bytes.start >= bytes.end
            || bytes.end > buffer.len_bytes()
        {
            return Vec::new();
        }
        let first = self.ends.partition_point(|end| *end <= bytes.start);
        let last = self
            .captures
            .partition_point(|capture| capture.bytes.start < bytes.end);
        let captures: Vec<_> = self.captures[first..last]
            .iter()
            .filter(|capture| capture.bytes.end > bytes.start)
            .take(65_537)
            .cloned()
            .collect();
        if captures.len() > 65_536 {
            Vec::new()
        } else {
            captures
        }
    }
}

/// The index and styles are installed together only after exact-stamp admission.
pub struct Coloring {
    pub(in crate::org) index: Index,
    pub(in crate::org) highlights: Highlights,
    outline: std::sync::Arc<super::Outline>,
}

impl Coloring {
    pub fn highlights(&self) -> &Highlights {
        &self.highlights
    }
    pub(crate) fn into_parts(self) -> (Index, Highlights, std::sync::Arc<super::Outline>) {
        (self.index, self.highlights, self.outline)
    }
}

/// Explicit owned analysis: COW rope snapshots, finite source/capture limits,
/// and the same cooperative cancellation contract as structural observations.
pub struct ColorScan {
    text: ropey::Rope,
    index: Scan,
    structure: StructureScan,
    limits: Limits,
    stamp: Stamp,
    index_start: usize,
}

impl ColorScan {
    pub fn new(buffer: &Buffer, limits: Limits) -> Result<Self, Error> {
        Self::with_index(buffer, Scan::new(buffer, limits)?, limits)
    }

    pub fn resume(index: Index, buffer: &Buffer) -> Result<Self, Error> {
        let limits = index.limits();
        Self::with_index(buffer, Scan::resume(index, buffer)?, limits)
    }

    fn with_index(buffer: &Buffer, index: Scan, limits: Limits) -> Result<Self, Error> {
        Ok(Self {
            index_start: index.scanned_bytes().0,
            index,
            text: Source::new(buffer).snapshot()?,
            structure: StructureScan::new(buffer, limits)?,
            limits,
            stamp: buffer.stamp(),
        })
    }

    pub fn run(self, cancellation: &Cancellation) -> Coloring {
        let index = self.index.run(cancellation);
        let structure = self.structure.run(cancellation);
        let mut highlights = Highlights {
            stamp: self.stamp,
            coverage: index.coverage(),
            captures: Vec::new(),
            ends: Vec::new(),
            reindexed: index.covered_bytes().0.saturating_sub(self.index_start),
        };
        if highlights.coverage == Coverage::Complete {
            highlights.coverage = structure.coverage();
        }
        build::project(
            &self.text,
            &structure,
            &mut highlights,
            self.limits,
            cancellation,
        );
        let outline = std::sync::Arc::new(super::Outline::observe(
            &structure,
            &self.text,
            cancellation,
        ));
        Coloring {
            index,
            highlights,
            outline,
        }
    }
}
