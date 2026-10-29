use super::*;
use crate::{coordinates::Coordinates, view::View};
impl Session {
    pub(in crate::session) fn finish_capture(&mut self, result: Result<Resolved, String>) {
        let Some(state) = self.capture.take() else {
            return;
        };
        let Stage::Waiting(text) = state.stage else {
            unreachable!("capture job owns waiting stage")
        };
        if !self.capture_current(&state.origin) {
            self.error("Capture origin changed during destination discovery");
            return;
        }
        match result.and_then(|target| self.install_capture(text).map(|id| (id, target))) {
            Ok((id, target)) => {
                self.capture = Some(State {
                    origin: state.origin,
                    stage: Stage::Draft(id, Box::new(target)),
                });
                self.sync_input_context();
                self.message("Capture draft: C-c C-c finalizes; C-c C-k aborts");
            }
            Err(e) => {
                self.error(&e);
            }
        }
    }
    fn install_capture(&mut self, text: DraftText) -> Result<BufferId, String> {
        let id = self
            .buffers
            .capture_draft(&text.text)
            .map_err(|e| e.to_string())?;
        let result = self.address_draft(id, text.cursor);
        if let Err(error) = result {
            self.discard_capture_buffer(id)?;
            return Err(error);
        }
        Ok(id)
    }
    fn address_draft(
        &mut self,
        id: BufferId,
        cursor: crate::coordinates::Byte,
    ) -> Result<(), String> {
        let buffer = self.buffers.get(id).map_err(|e| e.to_string())?;
        let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let mut point = coordinates.from_byte(cursor).map_err(|e| e.to_string())?;
        if !coordinates.is_boundary(point).map_err(|e| e.to_string())? {
            point = coordinates.next(point).map_err(|e| e.to_string())?;
        }
        let mut view = View::new(buffer);
        view.point.position = point;
        view.cell(buffer).map_err(|e| e.to_string())?;
        self.windows.retain_view(buffer, view.point)?;
        self.input.cancel();
        self.pending = None;
        Ok(())
    }
    pub(super) fn abort_capture(&mut self) -> Action {
        let Some(state) = self.capture.take() else {
            return self.message("No capture is active");
        };
        self.org.cancel_capture();
        self.input.cancel();
        self.pending = None;
        let published = matches!(state.stage, Stage::Published(_));
        if let Some(id) = state.stage.draft() {
            self.windows.restore_view();
            if let Err(e) = self.discard_capture_buffer(id) {
                return self.error(&e);
            }
        }
        if let Err(e) = self.windows.select(state.origin.view) {
            return self.error(&e.to_string());
        }
        self.sync_input_context();
        if published {
            self.message(
                "Capture draft discarded; already published source retained with its save status",
            )
        } else {
            self.message("Capture aborted; destination unchanged")
        }
    }
    pub(super) fn return_capture(&mut self, origin: Origin, id: BufferId) -> Action {
        self.windows.restore_view();
        if let Err(e) = self.discard_capture_buffer(id) {
            return self.error(&e);
        }
        if let Err(e) = self.windows.select(origin.view) {
            return self.error(&e.to_string());
        }
        self.sync_input_context();
        self.message("Capture finalized and saved")
    }
    pub(in crate::session) fn discard_capture_buffer(
        &mut self,
        id: BufferId,
    ) -> Result<(), String> {
        let kill = self.buffers.prepare_kill(id).map_err(|e| e.to_string())?;
        self.buffers.kill(kill).map_err(|e| e.to_string())?;
        Ok(())
    }
}
