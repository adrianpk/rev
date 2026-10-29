//! Synchronous at-most-two-source publication. All recoverable history and
//! position admission finishes before any participating canonical text changes.
use super::Session;
use crate::{
    buffer::{
        Buffer, BufferId,
        edit::{Point, Prepared, Replacement},
    },
    window::{Points, Windows},
};
pub(super) struct PreparedSource {
    pub(super) id: BufferId,
    prepared: Prepared,
    points: Points,
    tops: Vec<usize>,
}
impl PreparedSource {
    /// A special-edit draft retains its real source view outside the visible
    /// pane. Undo restores that source point rather than a peer or draft point.
    pub(super) fn prepare_retained(
        buffer: &mut Buffer,
        windows: &Windows,
        edits: &[Replacement],
    ) -> Result<Self, String> {
        let mut candidate = Self::prepare(buffer, windows, edits)?;
        let initiator = candidate
            .points
            .retained_initiator()
            .ok_or("Source edit home view is unavailable; draft retained")?;
        candidate
            .prepared
            .retain_initiator(initiator)
            .map_err(|e| e.to_string())?;
        Ok(candidate)
    }

    pub(super) fn prepare(
        buffer: &mut Buffer,
        windows: &Windows,
        edits: &[Replacement],
    ) -> Result<Self, String> {
        let points = windows.capture_points(buffer).map_err(|e| e.to_string())?;
        let mut prepared = buffer
            .prepare(edits, &points.values, None)
            .map_err(|e| e.to_string())?;
        if windows.selected().point.stamp.buffer == buffer.id() {
            prepared
                .retain_initiator(points.initiator)
                .map_err(|e| e.to_string())?;
        }
        prepared.set_origin(windows.selected_id());
        let tops = points.map_tops(&prepared).map_err(|e| e.to_string())?;
        buffer
            .admit_publication(&prepared, &points.values)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            id: buffer.id(),
            prepared,
            points,
            tops,
        })
    }
    pub(super) fn publish(mut self, buffer: &mut Buffer, windows: &mut Windows) {
        buffer
            .publish(self.prepared, &mut self.points.values)
            .expect("synchronous admitted source publication");
        windows.publish_points(self.points, self.tops);
    }
}
impl Session {
    pub(in crate::session) fn publish_clock_batch(
        &mut self,
        edits: &[Replacement],
        anchors: &[Point],
    ) -> Result<Vec<Point>, String> {
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
        let mut mapped = anchors.to_vec();
        for id in ids {
            let edits = edits
                .iter()
                .filter(|e| e.source.stamp.buffer == id)
                .cloned()
                .collect::<Vec<_>>();
            let buffer = self.buffers.get_mut(id).map_err(|e| e.to_string())?;
            let participant = PreparedSource::prepare(buffer, &self.windows, &edits)?;
            for anchor in mapped.iter_mut().filter(|a| a.stamp.buffer == id) {
                *anchor = participant
                    .prepared
                    .map_anchor(*anchor)
                    .map_err(|e| e.to_string())?;
            }
            batch.push(participant);
        }
        for participant in batch {
            let buffer = self
                .buffers
                .get_mut(participant.id)
                .expect("admitted clock participant");
            participant.publish(buffer, &mut self.windows);
        }
        self.windows.selected_mut().reset_navigation();
        Ok(mapped)
    }
}
