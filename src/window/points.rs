use crate::buffer::{Buffer, edit::Point};

use super::{ViewId, Windows};

/// Native edit owner maps every point for the selected canonical buffer.
pub(crate) struct Points {
    members: Vec<ViewId>,
    pub values: Vec<Point>,
    pub initiator: usize,
}

impl Windows {
    pub(crate) fn points(&self) -> Points {
        let buffer = self.selected().point.stamp.buffer;
        let (members, values): (Vec<_>, Vec<_>) = self
            .iter()
            .filter(|(_, view)| view.point.stamp.buffer == buffer)
            .map(|(id, view)| (id, view.point))
            .unzip();
        // The selected view belongs to this buffer and this private registry.
        let initiator = members
            .iter()
            .position(|id| *id == self.selected)
            .expect("selected point belongs to the captured set");
        Points {
            members,
            values,
            initiator,
        }
    }
    pub(crate) fn publish_points(&mut self, points: Points, buffer: &Buffer) {
        for (id, point) in points.members.into_iter().zip(points.values) {
            // No structural mutation occurs between synchronous capture/publication.
            let view = self
                .views
                .get_mut(&id)
                .expect("captured view remains live during publication");
            view.point = point;
            view.top = view.top.min(buffer.len_lines() - 1);
        }
    }
}
