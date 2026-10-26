use crate::command::History;

use super::{Action, Session};

impl Session {
    pub(super) fn history(&mut self, direction: History) -> Action {
        let result = self
            .buffers
            .get_mut(self.view.point.stamp.buffer)
            .map_err(crate::buffer::edit::Error::from)
            .and_then(|buffer| {
                buffer.navigate(
                    direction,
                    std::slice::from_mut(&mut self.view.point),
                    Some(0),
                )
            });
        self.finish(result.map(|()| true))
    }
}
