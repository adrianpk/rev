use super::*;
use crate::buffer::edit::Point;
use crate::command::{Command, OrgCommand as C};
use crate::org::time::TimestampField as Field;
use crate::window::ViewId;

pub(in crate::session) struct Repeat {
    view: ViewId,
    point: Point,
    command: C,
}
impl Repeat {
    fn current(&self, request: &Request) -> bool {
        self.view == request.view
            && self.point == request.origin.point
            && self.command == request.command
    }
}
impl Session {
    pub(in crate::session) fn dispatch_timestamp(
        &mut self,
        model: Arc<Outline>,
        mut request: Request,
    ) -> Result<(), String> {
        if request.command.timestamp() {
            return self.start_timestamp(model, request);
        }
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if !model.timestamp_context(buffer, request.origin.point.position)? {
            return Err("Org timestamp context is unavailable here".into());
        }
        if Target::at(buffer, request.origin.point.position)?.exists() {
            return self.start_timestamp(model, request);
        }
        match request.command {
            C::ShiftUp => {
                request.command = C::PriorityUp;
                self.start_estimate(model, request)
            }
            C::ShiftDown => {
                request.command = C::PriorityDown;
                self.start_estimate(model, request)
            }
            C::CtrlCCtrlC => self.timestamp_context_fallback(model, request),
            _ => self.timestamp_task_context(model, request),
        }
    }
    fn timestamp_context_fallback(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        if self.tag_request(&model, request.command) {
            return self.start_tags(model, request);
        }
        self.org_operation(model, request.command)?;
        self.message(request.command.name());
        Ok(())
    }
    fn timestamp_task_context(
        &mut self,
        model: Arc<Outline>,
        mut request: Request,
    ) -> Result<(), String> {
        if !model.timestamp_task_header(
            self.buffer().map_err(|e| e.to_string())?,
            request.origin.point.position,
        ) {
            return Err("No Org timestamp or task heading at point".into());
        }
        request.command = if request.command == C::ShiftRight {
            C::NextTodo
        } else {
            C::PreviousTodo
        };
        self.start_task(model, request)
    }
    pub(super) fn repeating_timestamp(&self, request: &Request) -> bool {
        self.timestamp_repeat
            .as_ref()
            .is_some_and(|repeat| repeat.current(request))
    }
    pub(in crate::session) fn timestamp_boundary(&mut self, command: Command) {
        if !matches!(
            command,
            Command::ExecuteExtended | Command::Org(C::Timestamp | C::TimestampInactive)
        ) {
            self.timestamp_repeat = None;
        }
    }
    pub(super) fn remember_timestamp(&mut self, prompt: &Prompt) {
        self.timestamp_repeat = None;
        if matches!(prompt.request.command, C::Timestamp | C::TimestampInactive)
            && prompt.target.repeats()
            && prompt.value.end.is_none()
        {
            self.timestamp_repeat = Some(Repeat {
                view: self.windows.selected_id(),
                point: self.view().point,
                command: prompt.request.command,
            });
        }
    }
    pub(super) fn adjust_timestamp(&mut self, mut prompt: Prompt) -> Result<Action, String> {
        if !prompt.target.exists() {
            return Err("No Org timestamp at point".into());
        }
        if prompt.request.command == C::CtrlCCtrlC {
            prompt.value.start.normalize_weekday();
        } else {
            let field = self.timestamp_field(&prompt)?;
            let up = matches!(
                prompt.request.command,
                C::TimestampUp | C::TimestampUpDay | C::ShiftUp | C::ShiftRight
            );
            prompt.value.start.step(field, up)?;
            if field == Field::Active {
                let active = prompt.value.start.active();
                if let Some(end) = &mut prompt.value.end {
                    end.set_active(active);
                }
            }
        }
        prompt.target.check(&prompt.value)?;
        self.validate_timestamp(prompt)
    }
    fn timestamp_field(&self, prompt: &Prompt) -> Result<Field, String> {
        match prompt.request.command {
            C::ShiftLeft | C::ShiftRight | C::TimestampUpDay | C::TimestampDownDay => {
                Ok(Field::Day)
            }
            _ => prompt.target.field(
                self.buffer().map_err(|e| e.to_string())?,
                prompt.request.origin.point.position,
            ),
        }
    }
}
pub(super) fn mode(value: &mut TimestampRange, command: C) {
    let active = match command {
        C::Timestamp | C::Schedule | C::Deadline | C::DateFromCalendar => true,
        C::TimestampInactive => false,
        _ => return,
    };
    value.start.set_active(active);
    if let Some(end) = &mut value.end {
        end.set_active(active);
    }
}
