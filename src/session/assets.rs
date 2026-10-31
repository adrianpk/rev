use super::{Action, Session};
use std::time::Duration;

impl Session {
    pub(super) fn assets_remaining(&self) -> Option<Duration> {
        self.buffers
            .assets_pending()
            .then_some(Duration::from_millis(10))
    }

    pub(super) fn expire_assets(&mut self) -> Action {
        if self.buffers.poll_assets() {
            self.invalidate_completion();
            Action::Redraw
        } else {
            Action::Wait
        }
    }
}
