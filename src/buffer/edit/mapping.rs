use std::ops::Range;

use super::{Buffer, Change, Cursor, Error, Point, Transaction};
use crate::coordinates::{Coordinates, Scalar};

#[derive(Clone, Copy)]
pub(super) enum Affinity {
    Left,
    Right,
}

pub(super) fn ordered(changes: &[Change], next: &Range<usize>) -> Result<(), Error> {
    if let Some(previous) = changes.last()
        && (next.start < previous.before.end || next.start == previous.before.start)
    {
        return Err(Error::Order);
    }
    Ok(())
}

pub(super) fn boundary(coordinates: &Coordinates<'_>, point: Scalar) -> Result<(), Error> {
    if !coordinates.is_boundary(point)? {
        return Err(crate::coordinates::Error::Cluster(point).into());
    }
    Ok(())
}

fn adjust(
    coordinates: &Coordinates<'_>,
    point: Scalar,
    affinity: Affinity,
) -> Result<Scalar, Error> {
    if coordinates.is_boundary(point)? {
        return Ok(point);
    }
    match affinity {
        Affinity::Left => Ok(coordinates.previous(point)?),
        Affinity::Right => Ok(coordinates.next(point)?),
    }
}

pub(super) fn mapped(
    changes: &[Change],
    mut point: Scalar,
    affinity: Affinity,
    permutation: Option<&super::Permutation>,
) -> Result<Scalar, Error> {
    if let Some(mapped) = permutation.and_then(|p| p.mapped(point)) {
        return Ok(mapped);
    }
    for change in changes.iter().rev() {
        point = through(change, point, affinity)?;
    }
    Ok(point)
}

fn through(change: &Change, point: Scalar, affinity: Affinity) -> Result<Scalar, Error> {
    let range = &change.before;
    if point.0 < range.start {
        return Ok(point);
    }
    let inserted = change.inserted.len_chars();
    if point.0 > range.end {
        let shifted = point.0 - range.end + range.start;
        return shifted
            .checked_add(inserted)
            .map(Scalar)
            .ok_or(Error::Capacity);
    }
    let end = range.start.checked_add(inserted).ok_or(Error::Capacity)?;
    if point.0 == range.end && !range.is_empty() {
        return Ok(Scalar(end));
    }
    match affinity {
        Affinity::Left => Ok(Scalar(range.start)),
        Affinity::Right => Ok(Scalar(end)),
    }
}

pub(super) fn map_points(
    buffer: &Buffer,
    changes: &[Change],
    points: &[Point],
    coordinates: &Coordinates<'_>,
    permutation: Option<&super::Permutation>,
) -> Result<Vec<Point>, Error> {
    let source = Coordinates::new(buffer)?;
    let mut mapped_points = Vec::with_capacity(points.len());
    for point in points {
        buffer.current(point.stamp)?;
        boundary(&source, point.position)?;
        let position = mapped(changes, point.position, Affinity::Right, permutation)?;
        mapped_points.push(Point {
            stamp: coordinates.stamp(),
            position: adjust(coordinates, position, Affinity::Right)?,
        });
    }
    Ok(mapped_points)
}

pub(super) fn map_mark(
    record: &Transaction,
    coordinates: &Coordinates<'_>,
) -> Result<Option<Scalar>, Error> {
    let Some(mark) = record.before_mark else {
        return Ok(None);
    };
    let mapped = mapped(
        &record.changes,
        mark,
        Affinity::Left,
        record.permutation.as_ref(),
    )?;
    Ok(Some(adjust(coordinates, mapped, Affinity::Left)?))
}

pub(super) fn set_cursor(
    record: &mut Transaction,
    cursor: Option<Cursor>,
    coordinates: &Coordinates<'_>,
) -> Result<(), Error> {
    if let Some(cursor) = cursor {
        let point = record
            .after_points
            .get_mut(cursor.index)
            .ok_or(Error::Cursor(cursor.index))?;
        point.position = adjust(coordinates, cursor.position, Affinity::Right)?;
        record.cursor = Some(cursor.index);
    }
    Ok(())
}
