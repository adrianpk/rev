use crate::buffer::edit::Error;
use crate::command::History;

use super::{Action, Session};

impl Session {
    pub(super) fn history(&mut self, direction: History) -> Action {
        let result = self.replay(direction);
        self.finish(result.map(|()| true))
    }

    fn replay(&mut self, direction: History) -> Result<(), Error> {
        let buffer = self
            .buffers
            .get_mut(self.windows.selected().point.stamp.buffer)?;
        let points = self.windows.points(buffer)?;
        let initiator = points.history_initiator(buffer, direction);
        let prepared = buffer.prepare_history(direction, &points.values, Some(initiator))?;
        self.windows.publish(buffer, prepared, points)?;
        Ok(())
    }
}
