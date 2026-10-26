use crate::buffer::{
    Buffer,
    edit::{Error, History, Point, Prepared},
};
use crate::coordinates::{Coordinates, LineByte};

use super::{ViewId, Windows};

/// A synchronous capture of every point and source viewport for one buffer.
/// Structural changes cannot occur between this capture and publication.
pub(crate) struct Points {
    members: Vec<(ViewId, Point)>,
    pub values: Vec<Point>,
    pub initiator: usize,
}

impl Windows {
    pub(crate) fn points(&self, buffer: &Buffer) -> Result<Points, Error> {
        let coordinates = Coordinates::new(buffer)?;
        let mut members = Vec::new();
        let mut values = Vec::new();
        for (id, view) in self
            .iter()
            .filter(|(_, view)| view.point.stamp.buffer == buffer.id())
        {
            let position = coordinates.from_line(LineByte {
                row: view.top,
                column: 0,
            })?;
            members.push((
                id,
                Point {
                    stamp: buffer.stamp(),
                    position,
                },
            ));
            values.push(view.point);
        }
        // The selected view belongs to this buffer and this private registry.
        let initiator = members
            .iter()
            .position(|(id, _)| *id == self.selected)
            .expect("selected point belongs to the captured set");
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

    fn publish_points(&mut self, points: Points, tops: Vec<usize>) {
        for (((id, _), point), top) in points.members.into_iter().zip(points.values).zip(tops) {
            let view = self
                .views
                .get_mut(&id)
                .expect("captured view remains live during publication");
            view.point = point;
            view.top = top;
        }
    }
}

impl Points {
    pub(crate) fn map_tops(&self, prepared: &Prepared) -> Result<Vec<usize>, Error> {
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
            .and_then(|origin| self.members.iter().position(|(id, _)| *id == origin))
            .unwrap_or(self.initiator)
    }
}
