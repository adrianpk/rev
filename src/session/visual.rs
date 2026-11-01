mod editing;
mod objects;
use super::Session;
use crate::buffer::Visual;
use crate::input::EditingState;
use crate::view::Error;

impl Session {
    pub(super) fn enter_visual(&mut self, shape: Visual) -> Result<(), Error> {
        let point = self.view().point;
        let buffer = self.buffers.get_mut(point.stamp.buffer)?;
        if buffer.visual() == Some(shape) {
            self.leave_visual();
            self.input.set_editing_state(EditingState::Normal);
            return Ok(());
        }
        buffer.activate_visual(point, shape)?;
        self.visual_source = Some(point.stamp);
        self.input
            .set_editing_state(EditingState::from_visual(shape));
        Ok(())
    }

    pub(super) fn leave_visual(&mut self) {
        if let Some(stamp) = self.visual_source.take()
            && let Ok(buffer) = self.buffers.get_mut(stamp.buffer)
            && buffer.visual().is_some()
        {
            buffer.deactivate_region();
        }
    }

    /// Only source identity/freshness is retained here. The authoritative shape,
    /// anchor and active endpoint belong to the native buffer region and view.
    pub(super) fn sync_visual_context(&mut self) {
        let Some(stamp) = self.visual_source else {
            return;
        };
        let shape = self
            .buffer()
            .ok()
            .filter(|buffer| buffer.stamp() == stamp)
            .and_then(|buffer| buffer.visual());
        if let Some(shape) = shape {
            let state = EditingState::from_visual(shape);
            if self.editing_state() != Some(state) {
                self.input.set_editing_state(state);
            }
        } else {
            self.leave_visual();
            self.input.set_editing_state(EditingState::Normal);
        }
    }
}
