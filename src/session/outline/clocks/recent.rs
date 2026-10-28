use super::*;

impl Session {
    pub(super) fn remember_clock(&mut self, point: Point) {
        let limit = self.buffers.snapshot().settings().org.clock.history_limit;
        self.recent_clocks
            .retain(|p| p.stamp.buffer != point.stamp.buffer || p.position != point.position);
        if limit > 0 {
            self.recent_clocks.push_front(point);
        }
        self.recent_clocks.truncate(limit);
    }
    pub(super) fn clock_recent_goto(&mut self) -> Action {
        match self.goto_recent_clock() {
            Ok(()) => self.message("Org recent clock task"),
            Err(error) => self.error(&error),
        }
    }
    pub(super) fn clock_in_last(&mut self) -> Action {
        match self.goto_recent_clock() {
            Ok(()) => self.org_command(OrgCommand::ClockIn),
            Err(error) => self.error(&error),
        }
    }
    fn goto_recent_clock(&mut self) -> Result<(), String> {
        let point = self
            .recent_clocks
            .front()
            .copied()
            .ok_or("No active or recent Org clock")?;
        let buffer = self
            .buffers
            .get(point.stamp.buffer)
            .map_err(|e| e.to_string())?;
        if buffer.stamp() != point.stamp {
            return Err("Recent Org clock source changed; select and check its task again".into());
        }
        self.activate(point.stamp.buffer);
        self.place_point(point).map_err(|e| e.to_string())
    }
    pub(super) fn adopt_clock(
        &mut self,
        mut active: Active,
        acknowledged: crate::org::time::OperationTime,
    ) -> Result<(), String> {
        active.acknowledged = acknowledged;
        self.remember_clock(active.owner);
        self.active_clock = Some(active);
        self.retained_clock = None;
        self.checkpoint_clock()
    }
}
