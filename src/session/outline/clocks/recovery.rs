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
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let index = model.tag_target(buffer, request.origin.point.position)?;
        let records = model.clock_observations(buffer, index)?;
        if records.is_empty() {
            return Err("No incomplete Org CLOCK in this entry".into());
        }
        let operation = self.org_clock.unwrap_or(super::super::timestamps::capture)(
            &self.buffers.snapshot().settings().org.time,
        )?;
        let recovery = Recovery {
            request,
            records,
            operation,
            stage: Stage::Record,
        };
        self.recovery_prompt(
            recovery,
            "Choose a 1-based incomplete clock number; C-g cancels",
        )
        .map(|_| ())
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
                self.active_clock = None;
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
        let record = &recovery.records[index];
        let civil = record
            .value
            .start
            .civil()
            .ok_or("Org CLOCK requires a source time")?;
        let result = recovery.operation.zone().resolve(civil)?;
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
            self.active_clock = Some(active);
            return Ok(self.message("Org clock resumed from checked source"));
        }
        recovery.stage = Stage::End(active);
        self.recovery_prompt(
            recovery,
            "Enter the accepted end date/time; excluded idle time is not counted; C-g cancels",
        )
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
        self.active_clock = Some(active);
        Ok(self.message("Org clock resumed with selected start offset"))
    }
    fn cancel_observed(&mut self, record: &ClockObservation) -> Result<Action, String> {
        let edit = Replacement {
            source: record.cleanup.as_ref().unwrap_or(&record.line).clone(),
            text: String::new(),
        };
        self.publish_clock_edit(edit)?;
        self.active_clock = None;
        Ok(self.message("Org incomplete interval cancelled"))
    }
}
fn resumed(record: &ClockObservation, operation: ZonedOperation) -> Result<Active, String> {
    let interval = ClockInterval::resume(&record.raw, operation)?;
    Ok(Active {
        source: record.line.clone(),
        owner: record.owner,
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
