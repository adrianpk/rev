use super::*;
impl Session {
    pub(super) fn edit_agenda_target(&mut self, run: &mut Run) {
        if run.index == run.targets.len() || self.org.busy() {
            return;
        }
        let Some(pin) = run.pins[run.index] else {
            run.index += 1;
            return;
        };
        let result = self.enter_agenda_source(pin);
        match result {
            Ok(stamp) => {
                run.replayed = false;
                run.before = Some(stamp);
                run.deadline = Instant::now()
                    + Duration::from_secs(
                        self.buffers.snapshot().settings().org.limits.job_seconds as u64,
                    );
                run.phase = Phase::Acting;
                self.org_command(run.command);
            }
            Err(e) => {
                run.outcome(run.index, ActionStatus::Rejected, e, None);
                run.index += 1;
            }
        }
    }
    fn enter_agenda_source(&mut self, pin: usize) -> Result<Stamp, String> {
        let point = self
            .windows
            .source_anchor(pin)
            .ok_or("Agenda source anchor unavailable")?;
        let buffer = self
            .buffers
            .get(point.stamp.buffer)
            .map_err(|e| e.to_string())?;
        if buffer.stamp() != point.stamp {
            return Err("Agenda source changed after preparation".into());
        }
        if buffer.is_readonly() {
            return Err("buffer is read-only".into());
        }
        self.windows.source_action(buffer, point)?;
        Ok(point.stamp)
    }
    pub(super) fn finish_agenda_target(&mut self, run: &mut Run) {
        if self.replay_agenda_argument(run) {
            return;
        }
        if self.pending.is_some() {
            return;
        }

        if self.org.request.is_some() {
            if Instant::now() < run.deadline {
                return;
            }
            self.org.cancel_request();
            self.error("Agenda structural source observation time limit reached");
        }
        let before = run.before.take().expect("active source stamp");
        let after = self.buffers.get(before.buffer).map(|b| b.stamp()).ok();
        let changed = after.is_some_and(|s| s != before);
        let failed = self.echo_kind == crate::display::Message::Error;
        let status = match (changed, failed) {
            (true, false) => ActionStatus::Changed,
            (true, true) => ActionStatus::ChangedWithFailure,
            (false, true) => ActionStatus::Rejected,
            (false, false) => ActionStatus::Unchanged,
        };
        run.outcome(run.index, status, self.echo.clone(), after);
        self.windows.restore_source_action();
        run.index += 1;
        run.phase = Phase::Edit;
    }
    pub(in crate::session) fn agenda_clock_action(
        &mut self,
        model: &crate::org::Outline,
        command: C,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let index = model.tag_target(buffer, self.view().point.position)?;
        if !self.clock_same_task(model, index)? {
            return Err("Selected agenda task does not own the active clock".into());
        }
        self.finish_clock(command)?;
        self.message(command.name());
        Ok(())
    }
}
