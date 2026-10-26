use crate::buffer::edit::Error;
use crate::command::History;

use super::{Action, Session};

impl Session {
    pub(super) fn history(&mut self, direction: History) -> Action {
        let result = self.replay(direction);
        self.finish(result.map(|()| true))
    }

    fn replay(&mut self, direction: History) -> Result<(), Error> {
        let mut points = self.windows.points();
        let buffer = self
            .buffers
            .get_mut(self.windows.selected().point.stamp.buffer)?;
        buffer.navigate(direction, &mut points.values, Some(points.initiator))?;
        self.windows.publish_points(points, buffer);
        Ok(())
    }
}
