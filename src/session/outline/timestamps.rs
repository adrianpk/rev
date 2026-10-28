mod calendar;
mod context;
pub(in crate::session) use context::Repeat;
pub(super) mod resolution;
use super::Request;
use crate::buffer::edit::{Cursor, Error};
use crate::command::OrgCommand;
use crate::org::Outline;
use crate::org::time::{Date, TimestampRange, ZoneEnvironment, ZoneSelection, ZonedOperation};
use crate::org::timestamp::Target;
use crate::session::{Action, Pending, Session};
use std::sync::Arc;
use std::time::SystemTime;

pub(in crate::session) type Clock = fn(&crate::config::Time) -> Result<ZonedOperation, String>;
pub(in crate::session) struct Prompt {
    request: Request,
    target: Target,
    operation: ZonedOperation,
    original: TimestampRange,
    value: TimestampRange,
    stage: Stage,
}
enum Stage {
    Date,
    Offset(resolution::Validation),
}
impl Session {
    /// Inject one operation observation without changing ambient process state.
    /// The retained zone/instant is reused through calendar and offset prompts.
    pub fn with_org_clock(mut self, clock: Clock) -> Self {
        self.org_clock = Some(clock);
        self
    }
    pub(super) fn start_timestamp(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        let target = self.timestamp_target(&model, &request)?;
        let policy = &self.buffers.snapshot().settings().org.time;
        let operation = self.org_clock.unwrap_or(capture)(policy)?;
        let mut original = target.template(
            operation.now(),
            request.command != OrgCommand::TimestampInactive,
        )?;
        context::mode(&mut original, request.command);
        let mut prompt = Prompt {
            request,
            target,
            operation,
            value: original.clone(),
            original,
            stage: Stage::Date,
        };
        if prompt.request.command.timestamp_step()
            || prompt.request.command == OrgCommand::CtrlCCtrlC
        {
            return self.adjust_timestamp(prompt).map(|_| ());
        }
        if prompt.request.command == OrgCommand::DateFromCalendar {
            prompt.value.start.set_date(
                self.calendar_date
                    .ok_or("Open the Org calendar and select a date first")?,
            );
            prompt.value.end = None;
            return self.validate_timestamp(prompt).map(|_| ());
        }
        let feedback = prompt.value.format();
        self.timestamp_prompt(
            prompt,
            "",
            &format!("{feedback}; S-arrows select day/week; Enter accepts; C-g cancels"),
        )
        .map(|_| ())
    }
    fn timestamp_target(&self, model: &Outline, request: &Request) -> Result<Target, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        self.timestamp_admitted(model, request)?;
        if self.repeating_timestamp(request) {
            return Target::append_end(buffer, request.origin.point.position);
        }
        match request.command {
            OrgCommand::Schedule => {
                model.planning_target(buffer, request.origin.point.position, "SCHEDULED")
            }
            OrgCommand::Deadline => {
                model.planning_target(buffer, request.origin.point.position, "DEADLINE")
            }
            OrgCommand::DateFromCalendar => {
                Target::insertion(buffer, request.origin.point.position)
            }
            _ => Target::at(buffer, request.origin.point.position),
        }
    }
    fn timestamp_admitted(&self, model: &Outline, request: &Request) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if request.command != OrgCommand::GotoCalendar && buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        if request.command.timestamp_step()
            && !model.timestamp_context(buffer, request.origin.point.position)?
        {
            return Err("Org timestamp context is unavailable here".into());
        }
        Ok(())
    }
    fn timestamp_prompt(
        &mut self,
        prompt: Prompt,
        initial: &str,
        feedback: &str,
    ) -> Result<Action, String> {
        match prompt.stage {
            Stage::Date => self.input.calendar_argument(initial),
            Stage::Offset(_) => self.input.argument("Org UTC offset choice: ", initial),
        }
        .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Timestamp(Box::new(prompt)));
        Ok(self.message(feedback))
    }
    pub(in crate::session) fn timestamp_argument(
        &mut self,
        mut prompt: Prompt,
        draft: &str,
    ) -> Action {
        if !self.org_request_current(&prompt.request) {
            return self.message("Org timestamp source or view changed; request cancelled");
        }
        let stage = std::mem::replace(&mut prompt.stage, Stage::Date);
        let result = match stage {
            Stage::Date => self.timestamp_entered(prompt, draft),
            Stage::Offset(validation) => self.timestamp_offset(prompt, validation, draft),
        };
        match result {
            Ok(action) => action,
            Err(error) => self.error(&error),
        }
    }
    fn timestamp_entered(&mut self, mut prompt: Prompt, draft: &str) -> Result<Action, String> {
        match prompt.original.enter(draft, prompt.operation.now()) {
            Ok(value) => prompt.value = value,
            Err(error) => return self.timestamp_prompt(prompt, draft, &error),
        }
        if let Err(error) = prompt.target.check(&prompt.value) {
            return self.timestamp_prompt(prompt, draft, &error);
        }
        if prompt.request.command == OrgCommand::GotoCalendar {
            self.calendar_date = Some(prompt.value.start.date());
            return Ok(self.message("Org calendar date selected"));
        }
        self.validate_timestamp(prompt)
    }
    fn validate_timestamp(&mut self, mut prompt: Prompt) -> Result<Action, String> {
        context::mode(&mut prompt.value, prompt.request.command);
        let validation =
            resolution::Validation::new(&prompt.target.for_resolution(&prompt.value)?)?;
        self.resolve_timestamp(prompt, validation)
    }
    fn timestamp_offset(
        &mut self,
        mut prompt: Prompt,
        mut validation: resolution::Validation,
        draft: &str,
    ) -> Result<Action, String> {
        if let Err(error) = validation.select(draft) {
            prompt.stage = Stage::Offset(validation);
            return self.timestamp_prompt(prompt, draft, &error);
        }
        self.resolve_timestamp(prompt, validation)
    }
    fn resolve_timestamp(
        &mut self,
        mut prompt: Prompt,
        mut validation: resolution::Validation,
    ) -> Result<Action, String> {
        match validation.advance(prompt.operation.zone()) {
            Ok(true) => self.publish_timestamp(prompt),
            Ok(false) => {
                let feedback = validation.choices();
                prompt.stage = Stage::Offset(validation);
                self.timestamp_prompt(prompt, "", &feedback)
            }
            Err(error) => {
                let initial = prompt.value.format();
                self.timestamp_prompt(prompt, &initial, &error)
            }
        }
    }
    fn publish_timestamp(&mut self, prompt: Prompt) -> Result<Action, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let Some(edit) = prompt.target.replacement(buffer, &prompt.value)? else {
            return Ok(self.message("Org timestamp unchanged"));
        };
        let cursor = ((edit.source.range.is_empty() || prompt.target.appending())
            && !matches!(
                prompt.request.command,
                OrgCommand::Schedule | OrgCommand::Deadline
            ))
        .then_some(crate::coordinates::Scalar(
            edit.source.range.start + edit.text.chars().count(),
        ));
        let cursor = cursor.or(prompt
            .target
            .cursor(&prompt.value, prompt.request.origin.point.position)?);
        let buffer = self
            .buffers
            .get_mut(edit.source.stamp.buffer)
            .map_err(|e| e.to_string())?;
        let points = self.windows.points(buffer).map_err(|e| e.to_string())?;
        let cursor = cursor.map(|position| Cursor {
            index: points.initiator,
            position,
        });
        let mut prepared = buffer
            .prepare(&[edit], &points.values, cursor)
            .map_err(|e| e.to_string())?;
        prepared.set_origin(self.windows.selected_id());
        self.windows
            .publish(buffer, prepared, points)
            .map_err(|e| e.to_string())?;
        self.windows.selected_mut().reset_navigation();
        self.calendar_date = Some(prompt.value.start.date());
        self.remember_timestamp(&prompt);
        Ok(self.message("Org timestamp changed"))
    }
    pub(in crate::session) fn refresh_timestamp(&mut self) -> Option<Action> {
        let Some(Pending::Timestamp(prompt)) = &self.pending else {
            return None;
        };
        if !matches!(prompt.stage, Stage::Date) {
            return None;
        }
        if !self.org_source_current(&prompt.request) {
            self.pending = None;
            self.input.cancel();
            return Some(self.message("Org timestamp source or view changed; request cancelled"));
        }
        let draft = self.input.prompt()?.text();
        let result = prompt
            .original
            .enter(draft, prompt.operation.now())
            .and_then(|value| {
                prompt.target.check(&value)?;
                Ok(value)
            });
        let message = match result {
            Ok(value) => {
                let message = value.format();
                if let Some(Pending::Timestamp(prompt)) = &mut self.pending {
                    prompt.value = value;
                }
                message
            }
            Err(error) => error,
        };
        Some(self.message(&message))
    }
    pub(in crate::session) fn timestamp_calendar(&self) -> Option<Date> {
        let Some(Pending::Timestamp(prompt)) = &self.pending else {
            return None;
        };
        matches!(prompt.stage, Stage::Date).then_some(prompt.value.start.date())
    }
    pub(in crate::session) fn complete_timestamp(&mut self) -> Action {
        let Some(Pending::Timestamp(prompt)) = &self.pending else {
            return self.message("No Org date input");
        };
        if !matches!(prompt.stage, Stage::Date) {
            return self.message("Enter the 1-based observed UTC offset choice; C-g cancels");
        }
        let draft = self.input.prompt().expect("date prompt").text();
        let value = match prompt.original.enter(draft, prompt.operation.now()) {
            Ok(value) => value,
            Err(error) => return self.error(&error),
        };
        match self.input.complete(&value.format()) {
            Ok(()) => self.input_changed(),
            Err(error) => self.error(&error.to_string()),
        }
    }
}
pub(super) fn capture(policy: &crate::config::Time) -> Result<ZonedOperation, String> {
    let selection = ZoneSelection::parse(&policy.zone)?;
    let env = ZoneEnvironment::system();
    ZonedOperation::capture(&selection, &env, || Ok(SystemTime::now()))
}
