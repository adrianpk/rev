use super::*;
use crate::org::time::{Timestamp, TimestampRange};

impl Session {
    pub(in crate::session::outline::tasks) fn repeat_repaired(
        &mut self,
        mut prompt: Prompt,
        mut transition: Transition,
        draft: &str,
    ) -> Result<Action, String> {
        match repaired(&transition, draft) {
            Ok(value) => {
                transition
                    .change
                    .repeat
                    .as_mut()
                    .ok_or("Missing repeat")?
                    .values[transition.index]
                    .value = value;
                self.resolve_repeat(prompt, transition)
            }
            Err(error) => {
                prompt.stage = Stage::Repair(Box::new(transition));
                self.task_prompt(prompt, "Org repeat date/time: ", &error)
            }
        }
    }
}
fn repaired(transition: &Transition, draft: &str) -> Result<TimestampRange, String> {
    let now = transition
        .operation
        .as_ref()
        .ok_or("Missing completion observation")?
        .now();
    let value = &transition
        .change
        .repeat
        .as_ref()
        .ok_or("Missing repeat")?
        .values[transition.index]
        .value;
    let candidate = value.enter(draft, now)?;
    if !same_range(value, &candidate)? {
        return Err(
            "Retain the repeat cookie, warning, active status and clock/range fields".into(),
        );
    }
    Ok(candidate)
}
fn same_range(left: &TimestampRange, right: &TimestampRange) -> Result<bool, String> {
    if !same_stamp(&left.start, &right.start)? {
        return Ok(false);
    }
    match (&left.end, &right.end) {
        (Some(a), Some(b)) => same_stamp(a, b),
        (None, None) => Ok(true),
        _ => Ok(false),
    }
}
fn same_stamp(left: &Timestamp, right: &Timestamp) -> Result<bool, String> {
    Ok(left.active() == right.active()
        && left.time().is_some() == right.time().is_some()
        && left.end_time().is_some() == right.end_time().is_some()
        && left.repeater()? == right.repeater()?
        && left.warning_delay()? == right.warning_delay()?)
}
