mod repair;
use super::*;
use crate::org::task::Log;
use crate::org::time::ZonedOperation;
use crate::session::outline::timestamps::{self, resolution::Validation};

pub(super) struct Transition {
    pub(super) change: TaskChange,
    pub(super) stamp: String,
    operation: Option<ZonedOperation>,
    index: usize,
}
impl Session {
    pub(super) fn task_transition(&self, change: TaskChange) -> Result<Transition, String> {
        let mut transition = Transition {
            change,
            stamp: String::new(),
            operation: None,
            index: 0,
        };
        if transition.change.repeat.is_some() {
            return self.repeat_transition(transition);
        }
        let logging = &transition.change.logging;
        if logging.closed || logging.state != Log::None || logging.note {
            transition.stamp = self.task_timestamp()?;
        }
        Ok(transition)
    }
    fn repeat_transition(&self, mut transition: Transition) -> Result<Transition, String> {
        let operation = self.org_clock.unwrap_or(timestamps::capture)(
            &self.buffers.snapshot().settings().org.time,
        )?;
        let repeat = transition
            .change
            .repeat
            .as_mut()
            .ok_or("Missing Org repeat transition")?;
        for value in &mut repeat.values {
            value.value = value
                .value
                .advance_repeat(operation.now())?
                .ok_or("Org recurrence lost its active repeater")?;
        }
        transition.stamp = operation.now().format_timestamp(false);
        transition.operation = Some(operation);
        Ok(transition)
    }
    pub(super) fn resolve_repeat(
        &mut self,
        prompt: Prompt,
        mut transition: Transition,
    ) -> Result<Action, String> {
        let count = transition
            .change
            .repeat
            .as_ref()
            .map_or(0, |r| r.values.len());
        while transition.index < count {
            let value = &transition
                .change
                .repeat
                .as_ref()
                .ok_or("Missing recurrence")?
                .values[transition.index]
                .value;
            let mut validation = Validation::new(value)?;
            let zone = transition
                .operation
                .as_ref()
                .ok_or("Missing completion observation")?
                .zone();
            match validation.advance(zone) {
                Ok(true) => transition.index += 1,
                result => return self.repeat_resolution(prompt, transition, validation, result),
            }
        }
        self.finish_transition(prompt, transition)
    }
    fn finish_transition(
        &mut self,
        mut prompt: Prompt,
        transition: Transition,
    ) -> Result<Action, String> {
        if transition.change.logging.note {
            prompt.stage = Stage::Note(Box::new(transition));
            return self.task_prompt(
                prompt,
                "Org state note: ",
                "One-line note; empty records time only; C-g cancels the entire transition",
            );
        }
        self.publish_task(&prompt.model, &transition.change, &transition.stamp, "")
    }
    fn repeat_resolution(
        &mut self,
        mut prompt: Prompt,
        transition: Transition,
        validation: Validation,
        result: Result<bool, String>,
    ) -> Result<Action, String> {
        match result {
            Ok(false) => {
                let message = validation.choices();
                prompt.stage = Stage::Offset(Box::new(transition), validation);
                self.task_prompt(prompt, "Org repeat UTC offset: ", &message)
            }
            Err(error) => {
                prompt.stage = Stage::Repair(Box::new(transition));
                self.task_prompt(
                    prompt,
                    "Org repeat date/time: ",
                    &format!(
                        "{error}; enter a valid date/time retaining timestamp fields; C-g cancels"
                    ),
                )
            }
            Ok(true) => unreachable!("completed resolution advances its range"),
        }
    }
    pub(super) fn repeat_offset(
        &mut self,
        mut prompt: Prompt,
        mut transition: Transition,
        mut validation: Validation,
        draft: &str,
    ) -> Result<Action, String> {
        if let Err(error) = validation.select(draft) {
            prompt.stage = Stage::Offset(Box::new(transition), validation);
            return self.task_prompt(prompt, "Org repeat UTC offset: ", &error);
        }
        let zone = transition
            .operation
            .as_ref()
            .ok_or("Missing completion observation")?
            .zone();
        match validation.advance(zone) {
            Ok(true) => {
                transition.index += 1;
                self.resolve_repeat(prompt, transition)
            }
            result => self.repeat_resolution(prompt, transition, validation, result),
        }
    }
}
