use super::*;

impl Session {
    pub(super) fn recovery_end(
        &mut self,
        mut recovery: Recovery,
        active: Active,
        draft: &str,
    ) -> Result<Action, String> {
        if draft.trim().is_empty() {
            let end = recovery.operation.instant();
            return self.accept_clock_end(recovery, active, end, draft);
        }
        match end_resolution(&recovery, draft) {
            Ok(Resolution::Unique(choice)) => {
                let instant = OperationTime::from_unix_seconds(choice.unix_seconds);
                self.accept_clock_end(recovery, active, instant, draft)
            }
            Ok(Resolution::Ambiguous(choices)) => {
                let feedback = offsets(&choices);
                recovery.stage = Stage::EndOffset(active, choices);
                self.recovery_prompt(recovery, &feedback)
            }
            result => {
                let error = result.err().unwrap_or_else(|| {
                    "Nonexistent local end time; enter a valid date/time".into()
                });
                recovery.stage = Stage::End(active);
                self.recovery_retry(recovery, &error, draft)
            }
        }
    }
    pub(super) fn recovery_end_offset(
        &mut self,
        mut recovery: Recovery,
        active: Active,
        choices: Vec<crate::org::time::ResolvedTime>,
        draft: &str,
    ) -> Result<Action, String> {
        let Some(choice) = number(draft, choices.len()).map(|i| choices[i]) else {
            recovery.stage = Stage::EndOffset(active, choices);
            return self.recovery_retry(
                recovery,
                "Choose an observed end offset number, or C-g",
                draft,
            );
        };
        self.accept_clock_end(
            recovery,
            active,
            OperationTime::from_unix_seconds(choice.unix_seconds),
            draft,
        )
    }
    fn accept_clock_end(
        &mut self,
        mut recovery: Recovery,
        active: Active,
        end: OperationTime,
        draft: &str,
    ) -> Result<Action, String> {
        let start = active.interval.started();
        // Entered source ends have minute precision; matching the start minute
        // is a zero interval even when persisted start seconds were retained.
        let end = if end < start
            && end.unix_seconds().div_euclid(60) == start.unix_seconds().div_euclid(60)
        {
            start
        } else {
            end
        };
        let error = if end > recovery.operation.instant() {
            Some("Org clock end is after the captured operation time")
        } else if end < active.interval.started() {
            Some("Org clock ends before it starts")
        } else {
            None
        };
        if let Some(error) = error {
            recovery.stage = Stage::End(active);
            return self.recovery_retry(recovery, error, draft);
        }
        self.close_observed(&active, end)
    }
    fn close_observed(&mut self, active: &Active, end: OperationTime) -> Result<Action, String> {
        let record = active.interval.close(end)?;
        let buffer = self
            .buffers
            .get(active.source.stamp.buffer)
            .map_err(|e| e.to_string())?;
        let remove = self.buffers.snapshot().settings().org.clock.remove_zero
            && record.duration.minutes() == 0.0;
        let edit = if remove {
            Replacement {
                source: active.cleanup.as_ref().unwrap_or(&active.source).clone(),
                text: String::new(),
            }
        } else {
            Replacement {
                source: active.source.clone(),
                text: format!("{}{}", record.source, buffer.newline()),
            }
        };
        self.publish_clock_edit(edit)?;
        self.active_clock = None;
        self.retained_clock = None;
        self.checkpoint_clock()?;
        Ok(self.message("Org incomplete interval closed at the accepted end"))
    }
}
fn end_resolution(recovery: &Recovery, draft: &str) -> Result<Resolution, String> {
    let original = TimestampRange::parse(&recovery.operation.now().format_timestamp(false))?;
    let parsed = original.enter(draft, recovery.operation.now())?;
    if parsed.end.is_some() || parsed.start.end_time().is_some() {
        return Err("Org clock end requires one instant".into());
    }
    if parsed.start.repeater()?.is_some() || parsed.start.warning_delay()?.is_some() {
        return Err("Org clock end cannot contain recurrence or delay data".into());
    }
    let civil = parsed
        .start
        .civil()
        .ok_or("Org clock end requires date and time")?;
    recovery.operation.zone().resolve(civil)
}
