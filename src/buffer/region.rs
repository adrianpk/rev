use super::{
    Buffer, Span,
    edit::{Error, Point},
};
use crate::coordinates::{Coordinates, Scalar};

impl Buffer {
    pub fn mark_active(&self) -> bool {
        self.mark_active
    }

    /// Store a qualified endpoint and activate this buffer's transient region.
    pub fn activate_mark(&mut self, point: Point) -> Result<(), Error> {
        self.current(point.stamp)?;
        self.set_mark(Some(point.position))?;
        self.mark_active = true;
        Ok(())
    }

    /// Cancel presentation without discarding the stored endpoint.
    pub fn deactivate_region(&mut self) {
        self.mark_active = false;
        self.break_typing();
    }

    /// Order the shared mark and a live view's point, including an empty region.
    pub fn region(&self, point: Point) -> Result<Option<Span>, Error> {
        self.current(point.stamp)?;
        boundary(self, point.position)?;
        let Some(mark) = self.mark.filter(|_| self.mark_active) else {
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
        self.set_mark(Some(point.position))?;
        self.mark_active = true;
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
