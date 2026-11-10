use std::path::Path;

use super::{Kind, Request, Target};
use crate::{
    ai::pi::{History, Operation},
    buffer::{Buffer, edit::Point},
    config::{AiCommand, AiContext, AiInput, Pi},
    input::Focus,
    session::{Action, Pending, Session},
};

pub(super) enum Capture {
    Builtin(Kind),
    Saved(AiCommand),
}

impl Capture {
    fn target(self, buffer: &Buffer, point: Point) -> Result<(Target, AiCommand), String> {
        match self {
            Self::Builtin(kind) => {
                let target = Target::capture(buffer, point, kind)?;
                let input = if target.output == crate::config::AiOutput::Insert {
                    AiInput::Point
                } else {
                    AiInput::Selection
                };
                let definition = AiCommand {
                    prompt: String::new(),
                    input,
                    context: AiContext::Directory,
                    output: target.output,
                    model: None,
                };
                Ok((target, definition))
            }
            Self::Saved(definition) => {
                Ok((Target::defined(buffer, point, &definition)?, definition))
            }
        }
    }
}

impl Session {
    pub(in crate::session) fn ai_defined(&mut self, name: crate::command::AiName) -> Action {
        if let Err(error) = self.ai_reap_idle() {
            return self.error(&error);
        }
        let Some(definition) = self
            .buffers
            .snapshot()
            .settings()
            .ai
            .commands
            .get(name.key())
            .cloned()
        else {
            return self.error("AI definition is unavailable in this session");
        };
        let instruction = definition.prompt.clone();
        match self.ai_capture(Capture::Saved(definition)) {
            Ok(request) => self.ai_submit(request, &instruction),
            Err(error) => self.error(&error),
        }
    }
    pub(in crate::session) fn ai_command(&mut self, kind: Kind) -> Action {
        if let Err(error) = self.ai_reap_idle() {
            return self.error(&error);
        }
        match self.ai_capture(Capture::Builtin(kind)) {
            Ok(request) if kind == Kind::Explain => {
                self.ai_submit(request, super::presentation::EXPLAIN)
            }
            Ok(request) => {
                let target = &request.target;
                let label = format!(
                    "AI {} {} scalars {}..{}: ",
                    target.output.name(),
                    self.buffer().map_or("source", |b| b.name()),
                    target.source.range.start,
                    target.source.range.end
                );
                if let Err(error) = self.input.argument(&label, "") {
                    return self.error(&error.to_string());
                }
                self.pending = Some(Pending::Ai(Box::new(request)));
                self.message(
                    "Enter an instruction for this captured target; guarded directory context; C-g cancels",
                )
            }
            Err(error) => self.error(&error),
        }
    }

    pub(super) fn ai_capture(&self, capture: Capture) -> Result<Request, String> {
        self.ai_capture_admission()?;
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let (target, definition) = capture.target(buffer, self.view().point)?;
        if buffer
            .slice(&target.source)
            .map_err(|e| e.to_string())?
            .len_bytes()
            > self.buffers.snapshot().settings().ai.pi.max_input_bytes
        {
            return Err("Pi input exceeds max_input_bytes".into());
        }
        let root = self.ai_root();
        let return_to = self
            .buffers
            .locations
            .lock()
            .expect("location owner poisoned")
            .capture(buffer, self.view().point)
            .map_err(|e| e.to_string())?;
        Ok(Request {
            conflict: None,
            source_name: buffer.name().to_owned(),
            language: buffer.language(self.buffers.snapshot()).to_owned(),
            target,
            definition,
            root,
            return_to,
            view: self.windows.selected_id(),
            home: self.view().clone(),
        })
    }

    fn ai_capture_admission(&self) -> Result<(), String> {
        if self.ai.cleanup_failed {
            return Err("Pi cleanup failed; this session cannot start another child".into());
        }
        if self.ai.operation.is_some() {
            return Err("Pi is already active or stopping; C-g cancels".into());
        }
        if self.ai.ready.is_some() {
            return Err("Apply or discard the current AI result before another invocation".into());
        }
        if self.input.focus() != Focus::Editing
            || self.tree.focused()
            || self.windows.retained_view_active()
            || self.pending.is_some()
        {
            return Err("Editing must own the target for AI commands".into());
        }
        Ok(())
    }

    pub(in crate::session) fn ai_argument(
        &mut self,
        request: Request,
        instruction: &str,
    ) -> Action {
        self.ai_submit(request, instruction)
    }

    fn ai_submit(&mut self, request: Request, instruction: &str) -> Action {
        let context = request.definition.context.name();
        match self.ai_start(request, instruction) {
            Ok(()) => self.message(&format!(
                "Pi starting: {context} context and isolated history; C-g cancels"
            )),
            Err(error) => self.error(&error),
        }
    }

    pub(super) fn ai_start(
        &mut self,
        mut request: Request,
        instruction: &str,
    ) -> Result<(), String> {
        instruction_admission(instruction)?;
        if self.ai.operation.is_some() || self.ai.ready.is_some() {
            return Err("Another AI invocation or result is pending".into());
        }
        let (root, message) = self.ai_envelope(&mut request, instruction)?;
        let epoch = self
            .ai
            .epoch
            .checked_add(1)
            .ok_or("Pi invocation identity exhausted")?;
        self.ai_reap_idle()?;
        let operation = self.ai_launch(&mut request, &root, message, epoch)?;
        self.ai.epoch = epoch;
        request.root = root;
        self.ai.source = Some(request);
        self.ai.accepted = false;
        self.ai.operation = Some(operation);
        Ok(())
    }

    fn ai_launch(
        &mut self,
        request: &mut Request,
        root: &Path,
        message: String,
        epoch: u64,
    ) -> Result<Operation, String> {
        let directory = request.definition.context == AiContext::Directory;
        let mut policy = self.buffers.snapshot().settings().ai.pi.clone();
        policy.model = request
            .definition
            .model
            .clone()
            .or_else(|| self.ai.model.clone())
            .or(policy.model);
        request.definition.model = policy.model.clone();
        let reset = self.ai.reset.as_ref().is_some_and(|(r, m, requested)| {
            r == root && *m == policy.model && epoch >= *requested
        });
        if self.ai.idle.is_some() {
            self.ai.admit_retained(&policy, root, directory, reset)?;
            self.ai.resume(message, epoch)
        } else if directory {
            Operation::retained(
                policy,
                root.to_owned(),
                message,
                epoch,
                History { state: None, reset },
            )
        } else {
            Operation::start(policy, root.to_owned(), message, epoch)
        }
    }
}

impl super::Work {
    fn admit_retained(
        &mut self,
        policy: &Pi,
        root: &Path,
        directory: bool,
        reset: bool,
    ) -> Result<(), String> {
        let idle = self.idle.as_ref().expect("retained Pi owner");
        if !directory || reset || !idle.matches(policy, root) {
            let idle = self.idle.take().expect("retained Pi owner");
            idle.cancel();
            self.operation = Some(idle);
            return Err("Pi root/model/grant boundary changed; retry after owned cleanup".into());
        }
        Ok(())
    }

    fn resume(&mut self, message: String, epoch: u64) -> Result<Operation, String> {
        let mut idle = self.idle.take().expect("admitted retained Pi owner");
        if let Err(error) = idle.resume(message, epoch) {
            idle.cancel();
            self.operation = Some(idle);
            return Err(error);
        }
        Ok(idle)
    }
}

fn instruction_admission(instruction: &str) -> Result<(), String> {
    if instruction.trim().is_empty() || instruction.len() > 16_384 || instruction.contains('\0') {
        return Err("AI instruction must contain 1..=16384 UTF-8 bytes without NUL".into());
    }
    Ok(())
}
