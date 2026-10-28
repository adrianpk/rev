mod build;

use std::ops::Range;
use std::sync::Arc;

use crate::buffer::{Buffer, Stamp};
use crate::coordinates::Scalar;
use crate::org::SourceRange;

use super::{Error, View};

/// Projection admission errors never authorize changing source or moving point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldError {
    Mode,
    Boundary(Scalar),
    Limit,
    PointHidden(Scalar),
}

impl std::fmt::Display for FoldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mode => f.write_str("fold projection requires native Org mode"),
            Self::Boundary(point) => write!(
                f,
                "fold boundary {} is not a logical line boundary",
                point.0
            ),
            Self::Limit => f.write_str("fold projection exceeds 4096 source ranges"),
            Self::PointHidden(point) => write!(f, "fold would hide current point {}", point.0),
        }
    }
}
impl std::error::Error for FoldError {}
impl From<FoldError> for Error {
    fn from(error: FoldError) -> Self {
        Self::Fold(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Hidden {
    rows: Range<usize>,
    removed: usize,
}

/// Shared immutable offsets make splitting/cloning a view cheap. Revealing a
/// range replaces only the requesting view's map, never another view's choice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Folds {
    stamp: Stamp,
    hidden: Arc<[Hidden]>,
}

impl Folds {
    fn source(&self, row: usize) -> usize {
        let index = self
            .hidden
            .partition_point(|h| h.rows.end - h.removed <= row);
        let removed = index.checked_sub(1).map_or(0, |i| self.hidden[i].removed);
        row.saturating_add(removed)
    }

    fn project(&self, row: usize) -> Option<usize> {
        let index = self.hidden.partition_point(|h| h.rows.end <= row);
        if self
            .hidden
            .get(index)
            .is_some_and(|h| h.rows.contains(&row))
        {
            return None;
        }
        let removed = index.checked_sub(1).map_or(0, |i| self.hidden[i].removed);
        Some(row - removed)
    }

    fn boundary(&self, row: usize) -> usize {
        let index = self.hidden.partition_point(|h| h.rows.end <= row);
        if let Some(h) = self.hidden.get(index).filter(|h| h.rows.contains(&row)) {
            h.rows.end - h.removed
        } else {
            self.project(row).expect("a visible row has a projection")
        }
    }

    fn removed(&self) -> usize {
        self.hidden.last().map_or(0, |h| h.removed)
    }
}

impl View {
    /// Install complete logical-line source ranges atomically. EOF may terminate
    /// an unterminated last line; empty ranges are ignored. Stale/foreign/partial-line ranges and hiding
    /// point are rejected before replacing this view's existing visibility.
    pub fn set_folds(&mut self, buffer: &Buffer, ranges: &[SourceRange]) -> Result<(), Error> {
        let point = self.cell(buffer)?;
        let folds = build::folds(buffer, ranges)?;
        if folds.project(point.row).is_none() {
            return Err(FoldError::PointHidden(self.point.position).into());
        }
        self.folds = Some(folds);
        self.reset_navigation();
        Ok(())
    }

    pub fn clear_folds(&mut self) -> bool {
        self.folds.take().is_some()
    }

    fn active_folds(&self, buffer: &Buffer) -> Option<&Folds> {
        self.folds
            .as_ref()
            .filter(|folds| buffer.is_org() && folds.stamp == buffer.stamp())
    }

    pub fn visible_lines(&self, buffer: &Buffer) -> usize {
        buffer.len_lines() - self.active_folds(buffer).map_or(0, Folds::removed)
    }

    /// Resolve one local screen row to its canonical logical source row. Hidden
    /// source never aliases a displayed row; EOF/overflow return no target.
    pub fn source_row(&self, buffer: &Buffer, local: usize) -> Option<usize> {
        if self.point.stamp != buffer.stamp() {
            return None;
        }
        self.source_at(buffer, self.projected_top(buffer).checked_add(local)?)
    }

    /// Resolve canonical source to a local screen row; hidden and above-viewport
    /// source has no screen position. Columns retain the existing glyph owner.
    pub fn screen_row(&self, buffer: &Buffer, source: usize) -> Option<usize> {
        if self.point.stamp != buffer.stamp() || source >= buffer.len_lines() {
            return None;
        }
        self.project_row(buffer, source)?
            .checked_sub(self.projected_top(buffer))
    }

    pub(crate) fn source_at(&self, buffer: &Buffer, row: usize) -> Option<usize> {
        let source = self
            .active_folds(buffer)
            .map_or(row, |folds| folds.source(row));
        (source < buffer.len_lines()).then_some(source)
    }

    pub(crate) fn projected_top(&self, buffer: &Buffer) -> usize {
        self.active_folds(buffer)
            .map_or(self.top, |folds| folds.boundary(self.top))
    }

    pub(super) fn project_row(&self, buffer: &Buffer, row: usize) -> Option<usize> {
        self.active_folds(buffer)
            .map_or(Some(row), |folds| folds.project(row))
    }

    pub(super) fn reveal_fold(&mut self, buffer: &Buffer, row: usize) {
        if self.active_folds(buffer).is_none() {
            self.clear_folds();
            return;
        }
        let folds = self.folds.as_ref().expect("active folds are installed");
        if folds.project(row).is_some() {
            return;
        }
        let ranges = folds
            .hidden
            .iter()
            .filter(|h| !h.rows.contains(&row))
            .map(|h| h.rows.clone());
        self.folds = Some(build::from_rows(buffer.stamp(), ranges));
    }
}
