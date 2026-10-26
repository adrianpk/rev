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
}

impl Buffer {
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
        self.mark_mode == Mode::Rectangle
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
        Ok(Some(self.span(
            mark.0.min(point.position.0)..mark.0.max(point.position.0),
        )?))
    }

    /// Return the stored endpoint as this view's point; other views are unchanged.
    pub fn exchange_mark(&mut self, point: Point) -> Result<Point, Error> {
        self.current(point.stamp)?;
        boundary(self, point.position)?;
        let mark = self.mark.ok_or(Error::Mark)?;
        let mode = self.mark_mode;
        self.set_mark(Some(point.position))?;
        self.mark_mode = if mode == Mode::Rectangle {
            mode
        } else {
            Mode::Linear
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
