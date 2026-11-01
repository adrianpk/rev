use super::{Action, Session};
use crate::buffer::{Buffer, Visit, edit::Point};
use crate::config::InteractionProfile;
use crate::coordinates::Coordinates;
use crate::jump::Record;
use crate::location::Anchor;
use crate::view::{Error, View};
mod once;
mod resolve;
pub(super) use once::Continuation;

impl Session {
    pub(super) fn prepare_jump_to(
        &self,
        destination: &Buffer,
        point: Point,
    ) -> Result<Option<Record>, Error> {
        self.prepare_jump_between(self.view().point, destination, point)
    }

    pub(super) fn prepare_jump_between(
        &self,
        origin: Point,
        destination: &Buffer,
        point: Point,
    ) -> Result<Option<Record>, Error> {
        if self.interaction_profile() != InteractionProfile::Vim || origin == point {
            return Ok(None);
        }
        let source = self.buffers.get(origin.stamp.buffer)?;
        if source.is_transient() || destination.is_transient() {
            return Ok(None);
        }
        let mut locations = self
            .buffers
            .locations
            .lock()
            .expect("location owner poisoned");
        Ok(Some(Record {
            origin: locations.capture(source, origin)?,
            destination: locations.capture(destination, point)?,
        }))
    }

    pub(super) fn publish_jump(&mut self, record: Option<Record>) {
        if let Some(record) = record {
            self.windows.selected_mut().jumps.record(record);
        }
    }

    pub(super) fn place_jump_view(&mut self, candidate: View) -> Result<(), Error> {
        let record = self.prepare_jump_to(self.buffer()?, candidate.point)?;
        self.place_view(candidate)?;
        self.publish_jump(record);
        Ok(())
    }

    pub(super) fn place_jump_point(&mut self, point: Point) -> Result<(), Error> {
        let mut candidate = self.view().clone();
        candidate.point = point;
        candidate.reset_navigation();
        self.place_jump_view(candidate)
    }

    pub(super) fn traverse_jump(&mut self, backward: bool, count: usize) -> Action {
        match self.apply_jump(backward, count) {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error),
        }
    }

    fn apply_jump(&mut self, backward: bool, count: usize) -> Result<(), String> {
        if self.input.focus() != crate::input::Focus::Editing || self.tree.focused() {
            return Err("A component owns input focus".into());
        }
        let current = self
            .buffers
            .locations
            .lock()
            .expect("location owner poisoned")
            .capture(self.buffer().map_err(|e| e.to_string())?, self.view().point)
            .map_err(|e| e.to_string())?;
        let (history, target) = self
            .view()
            .jumps
            .traversal(backward, count, current)
            .ok_or("No jump in that direction")?;
        let (visit, point) = self.resolve_jump(&target)?;
        let mut view = self.jump_view(&visit, point)?;
        self.admit_jump_target(&target, &visit)?;
        self.stop_typing().map_err(|e| e.to_string())?;
        self.buffers.publish_visit(visit);
        target.bind(point.stamp);
        view.jumps = history;
        *self.windows.selected_mut() = view;
        self.sync_input_context();
        Ok(())
    }
    fn admit_jump_target(&self, target: &Anchor, visit: &Visit) -> Result<(), String> {
        let source = target.source();
        if source.closed {
            self.admit_reopened_jump(
                &source,
                visit.buffer(&self.buffers).map_err(|e| e.to_string())?,
            )?;
        }
        Ok(())
    }

    fn jump_view(&self, visit: &Visit, point: Point) -> Result<View, String> {
        let buffer = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        let mut view = self.view().clone();
        if view.point.stamp.buffer != buffer.id() {
            view.rebind(buffer);
        }
        view.point = point;
        view.check_point(buffer).map_err(|e| e.to_string())?;
        view.reset_navigation();
        view.reveal(buffer, self.pane_size())
            .map_err(|e| e.to_string())?;
        Ok(view)
    }
}
