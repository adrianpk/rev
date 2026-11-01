use super::{
    Buffer, Span,
    edit::{Error, Point},
};
use crate::coordinates::{Coordinates, Scalar};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Inactive,
    Linear,
    Rectangle,
    Visual(Visual),
}

/// Native selection shape over the shared mark and a live view point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visual {
    Character,
    Line,
    Block,
}

impl Buffer {
    pub fn visual(&self) -> Option<Visual> {
        match self.mark_mode {
            Mode::Visual(shape) => Some(shape),
            _ => None,
        }
    }

    /// Convert a live selection without replacing its anchor. Fresh entry
    /// captures the checked native point; endpoints remain in the buffer owner.
    pub fn activate_visual(&mut self, point: Point, shape: Visual) -> Result<(), Error> {
        self.current(point.stamp)?;
        boundary(self, point.position)?;
        if self.visual().is_none() {
            self.activate_mark(point)?;
        }
        self.mark_mode = Mode::Visual(shape);
        self.break_typing();
        Ok(())
    }

    pub fn mark_active(&self) -> bool {
        self.mark_mode != Mode::Inactive
    }

    /// Store a qualified endpoint and activate this buffer's transient region.
    pub fn activate_mark(&mut self, point: Point) -> Result<(), Error> {
        self.current(point.stamp)?;
        self.set_mark(Some(point.position))?;
        self.mark_mode = Mode::Linear;
        Ok(())
    }

    pub fn rectangle_active(&self) -> bool {
        matches!(
            self.mark_mode,
            Mode::Rectangle | Mode::Visual(Visual::Block)
        )
    }

    /// Reuse an active mark, or start a new cell selection at the live point.
    pub fn toggle_rectangle(&mut self, point: Point) -> Result<(), Error> {
        self.current(point.stamp)?;
        boundary(self, point.position)?;
        if self.rectangle_active() {
            self.deactivate_region();
            return Ok(());
        }
        if !self.mark_active() {
            self.activate_mark(point)?;
        }
        self.mark_mode = Mode::Rectangle;
        self.break_typing();
        Ok(())
    }

    /// Cancel presentation without discarding the stored endpoint.
    pub fn deactivate_region(&mut self) {
        self.mark_mode = Mode::Inactive;
        self.break_typing();
    }

    /// Order the shared mark and a live view's point, including an empty region.
    pub fn region(&self, point: Point) -> Result<Option<Span>, Error> {
        self.current(point.stamp)?;
        boundary(self, point.position)?;
        let Some(mark) = self.mark.filter(|_| self.mark_active()) else {
            return Ok(None);
        };
        let start = mark.min(point.position);
        let end = mark.max(point.position);
        let coordinates = Coordinates::new(self)?;
        let range = match self.visual() {
            Some(Visual::Character) => start.0..coordinates.next_observed(end)?.0,
            Some(Visual::Line) => self.visual_lines(start, end)?,
            _ => start.0..end.0,
        };
        Ok(Some(self.span(range)?))
    }

    fn visual_lines(&self, start: Scalar, end: Scalar) -> Result<std::ops::Range<usize>, Error> {
        let coordinates = Coordinates::new(self)?;
        let first = coordinates.line_start(start)?;
        let row = coordinates.to_line(end)?.row + 1;
        let last = if row < self.len_lines() {
            coordinates.from_cell(crate::coordinates::Cell { row, column: 0 })?
        } else {
            coordinates.end()
        };
        Ok(first.0..last.0)
    }

    /// Return the stored endpoint as this view's point; other views are unchanged.
    pub fn exchange_mark(&mut self, point: Point) -> Result<Point, Error> {
        self.current(point.stamp)?;
        boundary(self, point.position)?;
        let mark = self.mark.ok_or(Error::Mark)?;
        let mode = self.mark_mode;
        self.set_mark(Some(point.position))?;
        self.mark_mode = if mode == Mode::Inactive {
            Mode::Linear
        } else {
            mode
        };
        Ok(Point {
            position: mark,
            ..point
        })
    }
}

fn boundary(buffer: &Buffer, position: Scalar) -> Result<(), Error> {
    if !Coordinates::new(buffer)?.is_boundary(position)? {
        return Err(crate::coordinates::Error::Cluster(position).into());
    }
    Ok(())
}
