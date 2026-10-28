mod end;
use super::Prompt;
use super::*;
use crate::org::{
    ClockObservation,
    time::{OperationTime, Resolution, TimestampRange, ZonedOperation},
};
use crate::session::outline::Request;

pub(in crate::session) struct Recovery {
    request: Request,
    records: Vec<ClockObservation>,
    operation: ZonedOperation,
    stage: Stage,
    stored: Option<crate::org::io::ClockState>,
}
enum Stage {
    Record,
    Action(usize),
    StartOffset(usize, bool, Vec<crate::org::time::ResolvedTime>),
    End(Active),
    EndOffset(Active, Vec<crate::org::time::ResolvedTime>),
}
impl Session {
    pub(in crate::session::outline) fn resolve_clocks(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        if self.active_clock.is_some() && self.clock_current().is_ok() {
            return Err(
                "An Org clock is running; clock out or cancel before resolving another interval"
                    .into(),
            );
        }
        let (records, stored) = self.clock_recovery_records(&model, &request)?;
        let operation = self.org_clock.unwrap_or(super::super::timestamps::capture)(
            &self.buffers.snapshot().settings().org.time,
        )?;
        let recovery = Recovery {
            request,
            records,
            operation,
            stage: Stage::Record,
            stored,
        };
        self.recovery_prompt(
            recovery,
            "Choose a 1-based incomplete clock number; C-g cancels",
        )
        .map(|_| ())
    }
    fn clock_recovery_records(
        &mut self,
        model: &Outline,
        request: &Request,
    ) -> Result<(Vec<ClockObservation>, Option<crate::org::io::ClockState>), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let index = model.tag_target(buffer, request.origin.point.position)?;
        let mut records = model.clock_observations(buffer, index)?;
        let stored = self.clock_recovery_pin.take();
        if let Some(pin) = &stored {
            self.pin_clock_records(&mut records, pin)?;
        }
        if records.is_empty() {
            return Err("No incomplete Org CLOCK in this entry".into());
        }
        Ok((records, stored))
    }
    fn pin_clock_records(
        &self,
        records: &mut Vec<ClockObservation>,
        pin: &crate::org::io::ClockState,
    ) -> Result<(), String> {
        self.verify_stored_source(pin)?;
        records.retain(|r| {
            r.line.range == (pin.line_start..pin.line_end)
                && r.owner.position.0 == pin.heading
                && r.raw == pin.source
        });
        Ok(())
    }
    fn recovery_prompt(&mut self, recovery: Recovery, feedback: &str) -> Result<Action, String> {
        self.recovery_retry(recovery, feedback, "")
    }
    fn recovery_retry(
        &mut self,
        recovery: Recovery,
        feedback: &str,
        draft: &str,
    ) -> Result<Action, String> {
        let label = match recovery.stage {
            Stage::Record => "Org incomplete clock: ",
            Stage::Action(_) => "Org clock action: ",
            Stage::StartOffset(_, _, _) | Stage::EndOffset(_, _) => "Org clock UTC offset: ",
            Stage::End(_) => "Org clock end: ",
        };
        let detail = if matches!(recovery.stage, Stage::Record) {
            format!(
                "{feedback}; {} interval(s) in this entry",
                recovery.records.len()
            )
        } else {
            feedback.into()
        };
        self.ask_clock_draft(Prompt::Recovery(Box::new(recovery)), label, &detail, draft)
    }
    pub(super) fn recover_clock(
        &mut self,
        mut recovery: Recovery,
        draft: &str,
    ) -> Result<Action, String> {
        if !self.org_source_current(&recovery.request) {
            return Err("Org clock source or view changed; recovery cancelled".into());
        }
        if let Some(pin) = &recovery.stored {
            self.verify_stored_source(pin)?;
        }
        let stage = std::mem::replace(&mut recovery.stage, Stage::Record);
        match stage {
            Stage::Record => self.recovery_record(recovery, draft),
            Stage::Action(index) => self.recovery_action(recovery, index, draft),
            Stage::StartOffset(index, closing, choices) => {
                self.recovery_start_offset(recovery, (index, closing, choices), draft)
            }
            Stage::End(active) => self.recovery_end(recovery, active, draft),
            Stage::EndOffset(active, choices) => {
                self.recovery_end_offset(recovery, active, choices, draft)
            }
        }
    }
    fn recovery_record(&mut self, mut recovery: Recovery, draft: &str) -> Result<Action, String> {
        let Some(index) = number(draft, recovery.records.len()) else {
            return self.recovery_retry(recovery, "Choose a valid interval number", draft);
        };
        recovery.stage = Stage::Action(index);
        self.recovery_prompt(recovery, "resume = adopt interval; close = select end; cancel = remove; keep = preserve open; replaces unresolved tracking; C-g cancels")
    }
    fn recovery_action(
        &mut self,
        mut recovery: Recovery,
        index: usize,
        draft: &str,
    ) -> Result<Action, String> {
        match draft.trim() {
            "keep" => {
                self.retained_clock = self.active_clock.take();
                Ok(self.message("Org interval remains open and unresolved"))
            }
            "cancel" => self.cancel_observed(&recovery.records[index]),
            "resume" | "close" => self.begin_source_clock(recovery, index, draft.trim() == "close"),
            _ => {
                recovery.stage = Stage::Action(index);
                self.recovery_retry(recovery, "Choose resume, close, cancel or keep", draft)
            }
        }
    }
    fn begin_source_clock(
        &mut self,
        mut recovery: Recovery,
        index: usize,
        closing: bool,
    ) -> Result<Action, String> {
        if recovery.stored.is_some() {
            return self.begin_stored_clock(recovery, index, closing);
        }
        let record = &recovery.records[index];
        let result = start_resolution(&recovery, index)?;
        let choices = result.candidates().to_vec();
        if choices.len() != 1 {
            if choices.is_empty() {
                recovery.stage = Stage::Action(index);
                return self.recovery_prompt(
                    recovery,
                    "Nonexistent source start time; correct source, cancel or keep; C-g cancels",
                );
            }
            let feedback = offsets(&choices);
            recovery.stage = Stage::StartOffset(index, closing, choices);
            return self.recovery_prompt(recovery, &feedback);
        }
        let active = resumed(
            record,
            ZonedOperation::from_resolved(choices[0], recovery.operation.zone().clone())?,
        )?;
        if active.interval.started() > recovery.operation.instant() {
            recovery.stage = Stage::Action(index);
            return self.recovery_prompt(
                recovery,
                "Start is after captured operation time; choose cancel or keep",
            );
        }
        if !closing {
            self.adopt_clock(active, recovery.operation.instant())?;
            return Ok(self.message("Org clock resumed from checked source"));
        }
        recovery.stage = Stage::End(active);
        self.recovery_prompt(
            recovery,
            "Enter the accepted end date/time; excluded idle time is not counted; C-g cancels",
        )
    }
    fn begin_stored_clock(
        &mut self,
        mut recovery: Recovery,
        index: usize,
        closing: bool,
    ) -> Result<Action, String> {
        let selected = self.stored_operation(&recovery);
        let active = selected.and_then(|operation| resumed(&recovery.records[index], operation));
        let active = match active {
            Ok(active) => active,
            Err(error) => {
                recovery.stage = Stage::Action(index);
                return self.recovery_prompt(recovery, &error);
            }
        };
        if closing {
            recovery.stage = Stage::End(active);
            return self.recovery_prompt(recovery, "Enter the accepted end; the saved unaccounted interval is not accepted automatically; C-g cancels");
        }
        self.adopt_clock(active, recovery.operation.instant())?;
        Ok(self.message("Org saved clock resumed after source and instant checks"))
    }
    fn stored_operation(&self, recovery: &Recovery) -> Result<ZonedOperation, String> {
        let record = recovery
            .stored
            .as_ref()
            .ok_or("Missing saved clock selection")?;
        if record.acknowledged_seconds > recovery.operation.instant().unix_seconds() {
            return Err("Saved Org clock is after the captured operation time".into());
        }
        let zone = recovery.operation.zone();
        let value = crate::org::time::ResolvedTime {
            unix_seconds: record.start_seconds,
            offset_seconds: zone.offset_at(record.start_seconds)?,
        };
        ZonedOperation::from_resolved(value, zone.clone())
    }
    fn recovery_start_offset(
        &mut self,
        mut recovery: Recovery,
        data: (usize, bool, Vec<crate::org::time::ResolvedTime>),
        draft: &str,
    ) -> Result<Action, String> {
        let (index, closing, choices) = data;
        let Some(choice) = number(draft, choices.len()).map(|n| choices[n]) else {
            recovery.stage = Stage::StartOffset(index, closing, choices);
            return self.recovery_retry(
                recovery,
                "Choose a valid observed start offset, or C-g",
                draft,
            );
        };
        let operation = ZonedOperation::from_resolved(choice, recovery.operation.zone().clone())?;
        let active = resumed(&recovery.records[index], operation)?;
        if closing {
            recovery.stage = Stage::End(active);
            return self.recovery_prompt(recovery, "Enter the accepted end date/time; C-g cancels");
        }
        if active.interval.started() > recovery.operation.instant() {
            recovery.stage = Stage::Action(index);
            return self.recovery_prompt(
                recovery,
                "Start is after the captured operation time; choose cancel or keep",
            );
        }
        self.adopt_clock(active, recovery.operation.instant())?;
        Ok(self.message("Org clock resumed with selected start offset"))
    }
    fn cancel_observed(&mut self, record: &ClockObservation) -> Result<Action, String> {
        let edit = Replacement {
            source: record.cleanup.as_ref().unwrap_or(&record.line).clone(),
            text: String::new(),
        };
        self.publish_clock_edit(edit)?;
        self.active_clock = None;
        self.retained_clock = None;
        self.checkpoint_clock()?;
        Ok(self.message("Org incomplete interval cancelled"))
    }
}
fn resumed(record: &ClockObservation, operation: ZonedOperation) -> Result<Active, String> {
    let interval = ClockInterval::resume(&record.raw, operation)?;
    Ok(Active {
        source: record.line.clone(),
        owner: record.owner,
        acknowledged: interval.started(),
        interval,
        cleanup: record.cleanup.clone(),
    })
}
fn number(draft: &str, count: usize) -> Option<usize> {
    draft
        .trim()
        .parse::<usize>()
        .ok()?
        .checked_sub(1)
        .filter(|i| *i < count)
}

fn offsets(choices: &[crate::org::time::ResolvedTime]) -> String {
    let text = choices
        .iter()
        .enumerate()
        .map(|(index, value)| format!("{}: UTC{:+} seconds", index + 1, value.offset_seconds))
        .collect::<Vec<_>>()
        .join("; ");
    format!("Choose an observed offset number: {text}; C-g cancels")
}

fn start_resolution(recovery: &Recovery, index: usize) -> Result<Resolution, String> {
    let civil = recovery.records[index]
        .value
        .start
        .civil()
        .ok_or("Org CLOCK requires a source time")?;
    recovery.operation.zone().resolve(civil)
}
