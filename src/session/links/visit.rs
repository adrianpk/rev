use super::*;
use crate::{coordinates::Scalar, org::links::Navigation};
impl Session {
    pub(in crate::session) fn finish_link_navigation(
        &mut self,
        result: Result<Navigation, String>,
    ) {
        let Some(request) = self.links.request.take() else {
            return;
        };
        if !self.org_source_current(&request)
            || self.input.focus() != Focus::Editing
            || self.pending.is_some()
        {
            self.error("Link source or view changed; nothing followed");
            return;
        }
        let result = result.and_then(|navigation| self.visit_link(navigation));
        if let Err(error) = result {
            self.error(&error);
        }
    }
    fn visit_link(&mut self, navigation: Navigation) -> Result<(), String> {
        let (visit, point) = navigation.prepare(&self.buffers)?;
        let buffer = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        let mut view = self.view().clone();
        if view.point.stamp.buffer != buffer.id() {
            view.rebind(buffer);
        }
        view.point = point;
        view.reset_navigation();
        view.reveal(buffer, self.pane_size())
            .map_err(|e| e.to_string())?;
        self.stop_typing().map_err(|e| e.to_string())?;
        self.push_link_mark(self.view().point);
        self.buffers.publish_visit(visit);
        *self.windows.selected_mut() = view;
        self.sync_input_context();
        self.message("Org link destination");
        Ok(())
    }
    pub(in crate::session) fn move_link_point(&mut self, byte: usize) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let point = Point {
            stamp: buffer.stamp(),
            position: Scalar(
                buffer
                    .byte_span(byte..byte)
                    .map_err(|e| e.to_string())?
                    .range
                    .start,
            ),
        };
        self.place_point(point).map_err(|e| e.to_string())
    }
    fn push_link_mark(&mut self, point: Point) {
        self.links.marks.push_front(point);
        self.links.marks.truncate(64);
    }
    pub(in crate::session) fn link_mark(&mut self) -> Action {
        self.push_link_mark(self.view().point);
        self.message("Org link return point stored")
    }
    pub(in crate::session) fn link_back(&mut self) -> Action {
        let result = self.restore_link_mark();
        match result {
            Ok(()) => self.message("Org link return point"),
            Err(error) => self.error(&error),
        }
    }
    fn restore_link_mark(&mut self) -> Result<(), String> {
        let point = *self.links.marks.front().ok_or("No Org link return point")?;
        let buffer = self
            .buffers
            .get(point.stamp.buffer)
            .map_err(|e| e.to_string())?;
        buffer
            .span(point.position.0..point.position.0)
            .map_err(|e| e.to_string())?;
        if buffer.stamp() != point.stamp {
            return Err("Stored return source changed; position is not current".into());
        }
        let mut view = self.view().clone();
        if view.point.stamp.buffer != buffer.id() {
            view.rebind(buffer);
        }
        view.point = point;
        view.reset_navigation();
        view.reveal(buffer, self.pane_size())
            .map_err(|e| e.to_string())?;
        let current = self.view().point;
        *self.windows.selected_mut() = view;
        self.links.marks.pop_front();
        self.links.marks.push_back(current);
        self.sync_input_context();
        Ok(())
    }
}
