use crate::buffer::{
    Buffer,
    edit::{Error, History, Point, Prepared},
};
use crate::coordinates::{Coordinates, LineByte, Scalar};

use super::{ViewId, Windows};

/// A mapped source viewport, independent of its eventual pane-width reflow.
pub(crate) struct Viewport {
    pub row: usize,
    pub position: Scalar,
}

/// A synchronous capture of every point and source viewport for one buffer.
/// Structural changes cannot occur between this capture and publication.
pub(crate) struct Points {
    members: Vec<(Member, Point)>,
    pub values: Vec<Point>,
    pub initiator: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Member {
    View(ViewId),
    Saved,
    Anchor(usize),
}

impl Windows {
    /// A preview can display another buffer while retaining its explicit source
    /// view. Include that source point in canonical history, map its viewport
    /// before publication, and return the mapped view for native presentation.
    pub(crate) fn publish_observed(
        &mut self,
        buffer: &mut Buffer,
        replacement: crate::buffer::edit::Replacement,
        mut home: crate::view::View,
        origin_view: ViewId,
    ) -> Result<crate::view::View, Error> {
        home.check_point(buffer).map_err(|e| match e {
            crate::view::Error::Buffer(e) => Error::Source(e),
            crate::view::Error::Coordinates(e) => Error::Coordinates(e),
            crate::view::Error::Edit(e) => e,
            _ => Error::ChangedPositions,
        })?;
        let coordinates = Coordinates::new(buffer)?;
        let top = Point {
            stamp: buffer.stamp(),
            position: viewport_position(&home, &coordinates, buffer.soft_wrap())?,
        };
        let mut points = self.capture_points(buffer)?;
        let origin = points.values.len();
        points.values.push(home.point);
        let mut prepared = buffer.prepare(&[replacement], &points.values, None)?;
        prepared.retain_initiator(origin)?;
        prepared.set_origin(origin_view);
        let home_top = prepared.map_top(top)?;
        let tops = points.map_tops(&prepared)?;
        buffer.publish(prepared, &mut points.values)?;
        home.point = points.values.pop().expect("observed initiating point");
        home.top = home_top.row;
        home.wrap_start = home_top.position;
        home.clear_folds();
        home.reset_navigation();
        self.publish_points(points, tops);
        Ok(home)
    }

    pub(crate) fn reload(
        &mut self,
        buffer: &mut Buffer,
        publication: crate::buffer::reload::Publication,
        mut points: Points,
    ) -> std::io::Result<bool> {
        let tops = publication
            .edit
            .as_ref()
            .map(|edit| points.map_tops(edit))
            .transpose()
            .map_err(std::io::Error::other)?;
        let changed = buffer.publish_reload(publication, &mut points.values)?;
        if let Some(tops) = tops {
            self.publish_points(points, tops);
        }
        Ok(changed)
    }
    pub(crate) fn points(&self, buffer: &Buffer) -> Result<Points, Error> {
        let points = self.capture_points(buffer)?;
        assert_eq!(
            self.selected().point.stamp.buffer,
            buffer.id(),
            "selected point belongs to the captured set"
        );
        Ok(points)
    }

    /// A hidden participant can have no initiating view. Its zero fallback is
    /// never used as a cursor; all actual shared views are still captured.
    pub(crate) fn capture_points(&self, buffer: &Buffer) -> Result<Points, Error> {
        let coordinates = Coordinates::new(buffer)?;
        let mut members = Vec::new();
        let mut values = Vec::new();
        for (id, view) in self
            .iter()
            .filter(|(_, view)| view.point.stamp.buffer == buffer.id())
        {
            let position = viewport_position(view, &coordinates, buffer.soft_wrap())?;
            members.push((
                Member::View(id),
                Point {
                    stamp: buffer.stamp(),
                    position,
                },
            ));
            values.push(view.point);
        }
        if let Some((_, view)) = &self.saved
            && view.point.stamp.buffer == buffer.id()
        {
            let position = viewport_position(view, &coordinates, buffer.soft_wrap())?;
            members.push((
                Member::Saved,
                Point {
                    stamp: buffer.stamp(),
                    position,
                },
            ));
            values.push(view.point);
        }
        for (index, point) in self
            .anchors
            .iter()
            .enumerate()
            .filter(|(_, p)| p.stamp.buffer == buffer.id())
        {
            members.push((
                Member::Anchor(index),
                Point {
                    stamp: buffer.stamp(),
                    position: crate::coordinates::Scalar(0),
                },
            ));
            values.push(*point);
        }
        // The selected view belongs to this buffer and this private registry.
        let initiator = members
            .iter()
            .position(|(id, _)| *id == Member::View(self.selected))
            .unwrap_or(0);
        Ok(Points {
            members,
            values,
            initiator,
        })
    }

    /// Finish every fallible position map before the buffer publishes text.
    pub(crate) fn publish(
        &mut self,
        buffer: &mut Buffer,
        prepared: Prepared,
        mut points: Points,
    ) -> Result<(), Error> {
        let tops = points.map_tops(&prepared)?;
        buffer.publish(prepared, &mut points.values)?;
        self.publish_points(points, tops);
        Ok(())
    }

    pub(crate) fn publish_points(&mut self, points: Points, tops: Vec<Viewport>) {
        for (((id, _), point), top) in points.members.into_iter().zip(points.values).zip(tops) {
            let view = match id {
                Member::View(id) => self.views.get_mut(&id).expect("captured live view"),
                Member::Saved => &mut self.saved.as_mut().expect("owned source action view").1,
                Member::Anchor(index) => {
                    self.anchors[index] = point;
                    continue;
                }
            };
            view.point = point;
            view.top = top.row;
            view.wrap_start = top.position;
            view.clear_folds();
        }
    }
}

impl Points {
    pub(crate) fn retained_initiator(&self) -> Option<usize> {
        self.members
            .iter()
            .position(|(member, _)| *member == Member::Saved)
    }

    pub(crate) fn map_tops(&self, prepared: &Prepared) -> Result<Vec<Viewport>, Error> {
        self.members
            .iter()
            .map(|(_, top)| prepared.map_top(*top))
            .collect()
    }

    /// Restore the original live view of this buffer, falling back to the
    /// invoking view when that identity has closed or now displays other text.
    pub(crate) fn history_initiator(&self, buffer: &Buffer, direction: History) -> usize {
        let record = match direction {
            History::Undo => buffer.history().back(),
            History::Redo => buffer.redo_history().back(),
        };
        record
            .and_then(|record| record.origin())
            .and_then(|origin| {
                self.members
                    .iter()
                    .position(|(id, _)| *id == Member::View(origin))
            })
            .unwrap_or(self.initiator)
    }
}

fn viewport_position(
    view: &crate::view::View,
    coordinates: &Coordinates<'_>,
    wrap: bool,
) -> Result<Scalar, Error> {
    let start = coordinates.from_line(LineByte {
        row: view.top,
        column: 0,
    })?;
    if !wrap {
        return Ok(start);
    }
    let position = view.wrap_start.clamp(start, coordinates.line_end(start)?);
    Ok(if coordinates.is_boundary(position)? {
        position
    } else {
        coordinates.previous(position)?
    })
}
