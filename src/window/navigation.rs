use super::{Area, Error, ViewId, Windows};
use crate::view::Size;

/// Geometry-based pane focus, shared by named commands and profile bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Down,
    Up,
    Right,
}

impl Windows {
    /// Select no target when presentation is suspended or no pane overlaps in
    /// the requested direction. Native screen order breaks equal-distance ties.
    pub fn adjacent(&self, direction: Direction, size: Size) -> Result<Option<ViewId>, Error> {
        self.get(self.selected)?;
        let Some(geometry) = self.geometry(size) else {
            return Ok(None);
        };
        let origin = geometry
            .panes()
            .iter()
            .find(|(id, _)| *id == self.selected)
            .map(|(_, area)| *area)
            .ok_or(Error::Missing(self.selected))?;
        Ok(geometry
            .panes()
            .iter()
            .filter_map(|(id, area)| score(origin, *area, direction).map(|score| (*id, score)))
            .min_by_key(|(_, score)| *score)
            .map(|(id, _)| id))
    }
}

fn axes(area: Area, horizontal: bool) -> (std::ops::Range<u16>, std::ops::Range<u16>) {
    let columns = area.column()..area.column() + area.width();
    let rows = area.row()..area.row() + area.height();
    if horizontal {
        (columns, rows)
    } else {
        (rows, columns)
    }
}

fn score(origin: Area, candidate: Area, direction: Direction) -> Option<(u16, u32)> {
    let horizontal = matches!(direction, Direction::Left | Direction::Right);
    let (main, cross) = axes(origin, horizontal);
    let (target, overlap) = axes(candidate, horizontal);
    if cross.start.max(overlap.start) >= cross.end.min(overlap.end) {
        return None;
    }
    let gap = if matches!(direction, Direction::Left | Direction::Up) {
        main.start.checked_sub(target.end)?
    } else {
        target.start.checked_sub(main.end)?
    };
    let center = u32::from(cross.start) + u32::from(cross.end);
    let other = u32::from(overlap.start) + u32::from(overlap.end);
    Some((gap, center.abs_diff(other)))
}
