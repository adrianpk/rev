//! Synchronous, at-most-two-buffer publication. Every source, point, viewport
//! and history allocation is admitted before the first canonical text changes.
use super::*;
use crate::buffer::{BufferId, edit::Prepared};
use crate::window::Points;

struct Participant {
    id: BufferId,
    prepared: Prepared,
    points: Points,
    tops: Vec<usize>,
}
impl Session {
    pub(super) fn publish_clock_batch(
        &mut self,
        edits: &[Replacement],
        anchor: Point,
    ) -> Result<Point, String> {
        let mut ids = edits
            .iter()
            .map(|e| e.source.stamp.buffer)
            .collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        if ids.len() > 2 {
            return Err("Org clock operation exceeds two source buffers".into());
        }
        let mut batch = Vec::new();
        let mut mapped = anchor;
        for id in ids {
            let edits = edits
                .iter()
                .filter(|e| e.source.stamp.buffer == id)
                .cloned()
                .collect::<Vec<_>>();
            let participant = self.prepare_clock_participant(id, &edits)?;
            if id == anchor.stamp.buffer {
                mapped = participant
                    .prepared
                    .map_anchor(anchor)
                    .map_err(|e| e.to_string())?;
            }
            batch.push(participant);
        }
        for mut participant in batch {
            self.buffers
                .get_mut(participant.id)
                .expect("admitted clock participant")
                .publish(participant.prepared, &mut participant.points.values)
                .expect("synchronous admitted clock publication");
            self.windows
                .publish_points(participant.points, participant.tops);
        }
        self.windows.selected_mut().reset_navigation();
        Ok(mapped)
    }
    fn prepare_clock_participant(
        &mut self,
        id: BufferId,
        edits: &[Replacement],
    ) -> Result<Participant, String> {
        let buffer = self.buffers.get_mut(id).map_err(|e| e.to_string())?;
        let points = self
            .windows
            .capture_points(buffer)
            .map_err(|e| e.to_string())?;
        let mut prepared = buffer
            .prepare(edits, &points.values, None)
            .map_err(|e| e.to_string())?;
        if self.windows.selected().point.stamp.buffer == id {
            prepared
                .retain_initiator(points.initiator)
                .map_err(|e| e.to_string())?;
        }
        prepared.set_origin(self.windows.selected_id());
        let tops = points.map_tops(&prepared).map_err(|e| e.to_string())?;
        buffer
            .admit_publication(&prepared, &points.values)
            .map_err(|e| e.to_string())?;
        Ok(Participant {
            id,
            prepared,
            points,
            tops,
        })
    }
}
