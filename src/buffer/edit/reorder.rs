use std::ops::Range;

use super::{Buffer, Error, History, Point, Prepared, Replacement, mapping};
use crate::buffer::Span;
use crate::coordinates::{Coordinates, Scalar};

#[derive(Clone, Debug)]
struct Piece {
    before: Range<usize>,
    after: Range<usize>,
}

/// A bijection over two or three contiguous original fragments. It carries no
/// replacement text or independent mutation authority; the transaction owns both.
#[derive(Clone, Debug)]
pub(super) struct Permutation {
    pieces: Vec<Piece>,
    endpoints: Option<[(usize, usize); 2]>,
}

impl Permutation {
    fn new(buffer: &Buffer, sources: &[Span]) -> Result<Self, Error> {
        let ranges = partition(buffer, sources)?;
        let mut start = ranges[0].start;
        let mut pieces = Vec::new();
        for source in sources {
            let end = start
                .checked_add(source.range.len())
                .ok_or(Error::Capacity)?;
            pieces.push(Piece {
                before: source.range.clone(),
                after: start..end,
            });
            start = end;
        }
        let endpoints = end_points(buffer, &pieces)?;
        Ok(Self { pieces, endpoints })
    }

    pub(super) fn position(&self, position: Scalar) -> Scalar {
        if let Some((_, after)) = self
            .endpoints
            .iter()
            .flatten()
            .find(|(before, _)| *before == position.0)
        {
            return Scalar(*after);
        }
        self.pieces
            .iter()
            .find(|piece| piece.before.contains(&position.0))
            .map_or(position, |piece| {
                Scalar(piece.after.start + (position.0 - piece.before.start))
            })
    }

    pub(super) fn replay(&self, direction: History) -> Self {
        let mut order = self.clone();
        if direction == History::Undo {
            for piece in &mut order.pieces {
                std::mem::swap(&mut piece.before, &mut piece.after);
            }
            if let Some(endpoints) = &mut order.endpoints {
                for (before, after) in endpoints {
                    std::mem::swap(before, after);
                }
            }
        }
        order
    }
}

// EOF and the preceding fragment's line end exchange their insertion anchors.
// Mapping both keeps the permutation invertible on positions as well as text.
fn end_points(buffer: &Buffer, pieces: &[Piece]) -> Result<Option<[(usize, usize); 2]>, Error> {
    let eof = buffer.len_chars();
    let tail = buffer.span(eof.saturating_sub(1)..eof)?;
    if buffer.slice(&tail)?.chars().next() == Some('\n') {
        return Ok(None);
    }
    let Some(owner) = pieces.iter().find(|piece| piece.before.end == eof) else {
        return Ok(None);
    };
    let Some(next) = pieces
        .iter()
        .find(|piece| piece.after.start == owner.after.end)
    else {
        return Ok(None);
    };
    Ok(Some([(eof, owner.after.end), (next.before.start, eof)]))
}

fn partition(buffer: &Buffer, sources: &[Span]) -> Result<Vec<Range<usize>>, Error> {
    if !(2..=3).contains(&sources.len()) {
        return Err(Error::Order);
    }
    let coordinates = Coordinates::new(buffer)?;
    let mut ranges = Vec::new();
    for source in sources {
        buffer.slice(source)?;
        mapping::boundary(&coordinates, Scalar(source.range.start))?;
        mapping::boundary(&coordinates, Scalar(source.range.end))?;
        ranges.push(source.range.clone());
    }
    ranges.sort_by_key(|range| range.start);
    if ranges.iter().any(Range::is_empty)
        || ranges.windows(2).any(|pair| pair[0].end != pair[1].start)
    {
        return Err(Error::Order);
    }
    Ok(ranges)
}

impl Buffer {
    /// Reorder a complete partition of existing source, preserving its bytes and
    /// mapping live points, marks and viewports through those same fragments.
    pub(crate) fn prepare_reorder(
        &self,
        sources: &[Span],
        points: &[Point],
    ) -> Result<Prepared, Error> {
        self.admit(self.stamp())?;
        let order = Permutation::new(self, sources)?;
        let start = order
            .pieces
            .iter()
            .map(|p| p.before.start)
            .min()
            .ok_or(Error::Order)?;
        let end = order
            .pieces
            .iter()
            .map(|p| p.before.end)
            .max()
            .ok_or(Error::Order)?;
        let mut text = String::new();
        for source in sources {
            for chunk in self.slice(source)?.chunks() {
                text.push_str(chunk);
            }
        }
        let edit = Replacement {
            source: self.span(start..end)?,
            text,
        };
        self.prepare_ordered(&[edit], points, None, Some(order))
    }
}
