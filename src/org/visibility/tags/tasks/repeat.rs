mod observe;
mod planning;
use super::*;
use crate::buffer::Span;
use crate::org::time::TimestampRange;

pub(crate) struct RepeatChange {
    pub state: String,
    pub values: Vec<RepeatSource>,
    pub record: bool,
    pub habit: bool,
}
pub(crate) struct RepeatSource {
    pub source: Span,
    pub value: TimestampRange,
}
impl TaskChange {
    pub(crate) fn result_state(&self) -> &str {
        self.repeat.as_ref().map_or(&self.next, |r| &r.state)
    }
}
impl Outline {
    pub(super) fn task_repeat(
        &self,
        buffer: &Buffer,
        change: &mut TaskChange,
        policy: &crate::config::Todo,
    ) -> Result<(), String> {
        if !change.workflow.state(&change.next).is_some_and(|s| s.done) {
            return Ok(());
        }
        let habit = self.habit_record(buffer, change.index, policy)?.is_some();
        let values = observe::timestamps(self, buffer, change.index)?;
        if values.is_empty() {
            return Ok(());
        }
        let state = self.repeat_state(buffer, change)?;
        change.habit_reset(habit, &state)?;
        let log = self.repeat_logging(buffer, change.index, policy)?;
        change.logging.closed = false;
        if log == Log::Note {
            change.logging.note = true;
        }
        if change.logging.state == Log::None {
            change.logging.state = log;
        }
        change.repeat = Some(RepeatChange {
            state,
            values,
            record: log != Log::None,
            habit,
        });
        Ok(())
    }
    fn repeat_state(&self, buffer: &Buffer, change: &TaskChange) -> Result<String, String> {
        let state = self.task_property(buffer, change.index, "REPEAT_TO_STATE")?;
        if !state.is_empty() {
            if change.workflow.state(&state).is_none() {
                return Err("Unknown Org REPEAT_TO_STATE keyword".into());
            }
            return Ok(state);
        }
        let sequence = change
            .workflow
            .state(&change.old)
            .or_else(|| change.workflow.state(&change.next))
            .ok_or("Missing Org workflow state")?
            .sequence;
        Ok(change
            .workflow
            .states
            .iter()
            .find(|s| s.sequence == sequence && !s.done)
            .map_or_else(String::new, |s| s.name.clone()))
    }
    fn repeat_logging(
        &self,
        buffer: &Buffer,
        index: usize,
        policy: &crate::config::Todo,
    ) -> Result<Log, String> {
        use crate::config::RepeatLogging;
        let base = match policy.log_repeat {
            RepeatLogging::None => Log::None,
            RepeatLogging::Time => Log::Time,
            RepeatLogging::Note => Log::Note,
        };
        let file = self.tags.repeat_log.unwrap_or(base);
        let local = self.task_logging_property(buffer, index, "LOGGING")?;
        Ok(repeat_log(&local).unwrap_or(file))
    }
    pub(super) fn repeat_edits(
        &self,
        buffer: &Buffer,
        change: &TaskChange,
        stamp: &str,
        plan: &mut edit::Plan,
    ) -> Result<(), String> {
        let Some(repeat) = &change.repeat else {
            return Ok(());
        };
        planning::edit(self, buffer, change, plan)?;
        if repeat.record {
            let point = Coordinates::new(buffer)
                .map_err(|e| e.to_string())?
                .from_byte(crate::coordinates::Byte(self.headings[change.index].start))
                .map_err(|e| e.to_string())?;
            let owner = self.property_target(buffer, point)?;
            for value in self.property_edit(buffer, owner, "LAST_REPEAT", Some(stamp))? {
                plan.import(buffer, value)?;
            }
        }
        Ok(())
    }
}
pub(in crate::org::visibility::tags) fn repeat_log(text: &str) -> Option<Log> {
    text.split_whitespace()
        .filter_map(|token| match token {
            "nil" | "nologrepeat" => Some(Log::None),
            "logrepeat" => Some(Log::Time),
            "lognoterepeat" => Some(Log::Note),
            _ => None,
        })
        .next_back()
}
