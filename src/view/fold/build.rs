use std::ops::Range;

use crate::buffer::{Buffer, Stamp};
use crate::coordinates::{Coordinates, Scalar};
use crate::org::{Source, SourceRange};
use crate::view::{Error, FoldError};

use super::{Folds, Hidden};

pub(super) fn folds(buffer: &Buffer, ranges: &[SourceRange]) -> Result<Folds, Error> {
    if !buffer.is_org() {
        return Err(FoldError::Mode.into());
    }
    if ranges.len() > 4096 {
        return Err(FoldError::Limit.into());
    }
    let mut rows = Vec::with_capacity(ranges.len());
    for range in ranges {
        Source::new(buffer).text(range)?;
        let source = &range.span().range;
        if source.is_empty() {
            continue;
        }
        let start = boundary(buffer, Scalar(source.start), false)?;
        let end = boundary(buffer, Scalar(source.end), true)?;
        rows.push(start..end);
    }
    rows.sort_unstable_by_key(|range| range.start);
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(rows.len());
    for row in rows {
        append(&mut merged, row);
    }
    Ok(from_rows(buffer.stamp(), merged))
}

fn boundary(buffer: &Buffer, point: Scalar, end: bool) -> Result<usize, Error> {
    let line = Coordinates::new(buffer)?.to_line(point)?;
    if line.column == 0 {
        return Ok(line.row);
    }
    if end && point.0 == buffer.len_chars() {
        return Ok(line.row + 1);
    }
    Err(FoldError::Boundary(point).into())
}

fn append(rows: &mut Vec<Range<usize>>, row: Range<usize>) {
    if let Some(previous) = rows.last_mut().filter(|previous| previous.end >= row.start) {
        previous.end = previous.end.max(row.end);
    } else {
        rows.push(row);
    }
}

pub(super) fn from_rows(stamp: Stamp, rows: impl IntoIterator<Item = Range<usize>>) -> Folds {
    let mut removed = 0;
    let hidden = rows
        .into_iter()
        .map(|rows| {
            removed += rows.len();
            Hidden { rows, removed }
        })
        .collect();
    Folds { stamp, hidden }
}
