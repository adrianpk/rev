use super::*;
use crate::{
    buffer::BufferId,
    coordinates::{Byte, Coordinates},
    session::source_batch::PreparedSource,
};
impl Session {
    pub(super) fn apply_feed(&mut self, plan: Plan) -> Result<(), String> {
        plan.current(&self.buffers)?;
        let state = self.feeds.as_ref().ok_or("Feed selection cancelled")?;
        let inbox = state.inbox;
        let visit = plan.insertion.source.prepare_visit(&self.buffers)?;
        if inbox {
            self.open_feed_inbox(plan, visit)
        } else {
            self.save_feed(plan, visit)
        }
    }
    fn open_feed_inbox(
        &mut self,
        plan: Plan,
        mut visit: crate::buffer::Visit,
    ) -> Result<(), String> {
        let buffer = visit
            .buffer_mut(&mut self.buffers)
            .map_err(|e| e.to_string())?;
        let edits = plan.insertion.replacements(buffer)?;
        if !edits.is_empty() {
            let prepared = PreparedSource::prepare(buffer, &self.windows, &edits)?;
            prepared.publish(buffer, &mut self.windows);
        }
        let id = self.buffers.publish_visit(visit);
        self.feed_inbox(id, plan.point)?;
        self.record_feed(&plan, None, "Inbox opened; changes remain in buffer".into());
        Ok(())
    }
    fn save_feed(&mut self, plan: Plan, mut visit: crate::buffer::Visit) -> Result<(), String> {
        let buffer = visit
            .buffer_mut(&mut self.buffers)
            .map_err(|e| e.to_string())?;
        let edits = plan.insertion.replacements(buffer)?;
        if edits.is_empty() {
            let id = self.buffers.publish_visit(visit);
            let outcome = self.buffers.save(id).map_err(|e| e.to_string())?;
            self.feed_saved(&plan, &outcome);
            return Ok(());
        }
        let mut points = self
            .windows
            .capture_points(buffer)
            .map_err(|e| e.to_string())?;
        let mut prepared = buffer
            .prepare(&edits, &points.values, None)
            .map_err(|e| e.to_string())?;
        prepared.set_origin(self.windows.selected_id());
        let tops = points.map_tops(&prepared).map_err(|e| e.to_string())?;
        let outcome = buffer
            .publish_saved_edit(prepared, &mut points.values, &mut crate::file::Files)
            .map_err(|e| e.to_string())?;
        if outcome.publication() != Publication::NotPublished {
            self.buffers.publish_visit(visit);
            self.windows.publish_points(points, tops);
        }
        self.feed_saved(&plan, &outcome);
        Ok(())
    }
    fn feed_saved(&mut self, plan: &Plan, outcome: &crate::file::Outcome) {
        let detail = outcome
            .error()
            .map(ToString::to_string)
            .unwrap_or("Feed destination saved".into());
        self.record_feed(plan, Some(outcome.publication()), detail);
    }
    fn record_feed(&mut self, plan: &Plan, publication: Option<Publication>, detail: String) {
        let Some(choice) = self.feeds.as_mut().and_then(|s| s.active.take()) else {
            return;
        };
        self.feed_results.push(FeedOutcome {
            name: choice.definition.name,
            path: choice.path,
            added: if publication == Some(Publication::NotPublished) {
                0
            } else {
                plan.added
            },
            skipped: plan.skipped,
            publication,
            detail,
        });
    }
    fn feed_inbox(&mut self, id: BufferId, point: usize) -> Result<(), String> {
        let buffer = self.buffers.get(id).map_err(|e| e.to_string())?;
        let mut view = self.view().clone();
        view.rebind(buffer);
        view.point.position = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .from_byte(Byte(point))
            .map_err(|e| e.to_string())?;
        view.reset_navigation();
        view.reveal(buffer, self.pane_size())
            .map_err(|e| e.to_string())?;
        *self.windows.selected_mut() = view;
        self.sync_input_context();
        Ok(())
    }
}
