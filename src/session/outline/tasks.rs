mod clock;

use super::Request;
use crate::buffer::edit::Error;
use crate::command::OrgCommand;
use crate::org::task::{Log, Workflow};
use crate::org::{Outline, TaskChange};
use crate::session::{Action, Pending, Session};
use std::sync::Arc;

pub(in crate::session) struct Prompt {
    request: Request,
    model: Arc<Outline>,
    index: usize,
    stage: Stage,
}
enum Stage {
    State(Workflow),
    Note(TaskChange, String),
}
impl Session {
    /// Supply an operation clock without changing process environment or source.
    /// The provider is called once for a logging transition, before its note prompt.
    pub fn with_task_clock(mut self, clock: fn() -> Result<String, String>) -> Self {
        self.task_clock = Some(clock);
        self
    }
    pub(super) fn start_task(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let index = model.tag_target(buffer, self.view().point.position)?;
        let workflow = model.task_workflow(&self.buffers.snapshot().settings().org.todo)?;
        let choices = choices(&workflow);
        if request.command == OrgCommand::InspectTodo {
            self.message(&format!(
                "Org task: {}; {choices}",
                workflow
                    .state(model.task_word(index))
                    .map_or("unmarked", |s| s.name.as_str())
            ));
            return Ok(());
        }
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let selection = request.command == OrgCommand::SetTodo
            || request.command == OrgCommand::Todo
                && workflow.states.iter().any(|s| s.key.is_some());
        if selection {
            let prompt = Prompt {
                request,
                model,
                index,
                stage: Stage::State(workflow),
            };
            return self
                .task_prompt(prompt, "Org task state: ", &choices)
                .map(|_| ());
        }
        let next = workflow.next(
            model.task_word(index),
            request.command == OrgCommand::PreviousTodo,
        );
        self.task_selected(
            Prompt {
                request,
                model,
                index,
                stage: Stage::State(workflow),
            },
            &next,
        )
        .map(|_| ())
    }
    fn task_prompt(
        &mut self,
        prompt: Prompt,
        label: &'static str,
        feedback: &str,
    ) -> Result<Action, String> {
        self.input.argument(label, "").map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Task(Box::new(prompt)));
        Ok(self.message(feedback))
    }
    pub(in crate::session) fn task_argument(&mut self, prompt: Prompt, draft: &str) -> Action {
        if !self.org_request_current(&prompt.request) {
            return self.message("Org task source or view changed; request cancelled");
        }
        let result = match &prompt.stage {
            Stage::State(workflow) => {
                select(workflow, draft).and_then(|next| self.task_selected(prompt, &next))
            }
            Stage::Note(change, stamp) => self.publish_task(&prompt.model, change, stamp, draft),
        };
        match result {
            Ok(action) => action,
            Err(error) => self.error(&error),
        }
    }
    fn task_selected(&mut self, mut prompt: Prompt, next: &str) -> Result<Action, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let policy = &self.buffers.snapshot().settings().org.todo;
        let change = prompt
            .model
            .task_change(buffer, prompt.index, next, policy)?;
        if change.old == change.next {
            return Ok(self.message("Org task state unchanged"));
        }
        let stamp =
            if change.logging.closed || change.logging.state != Log::None || change.logging.note {
                self.task_timestamp()?
            } else {
                String::new()
            };
        if change.logging.note {
            prompt.stage = Stage::Note(change, stamp);
            return self.task_prompt(
                prompt,
                "Org state note: ",
                "One-line note; empty records time only; C-g cancels the entire transition",
            );
        }
        self.publish_task(&prompt.model, &change, &stamp, "")
    }
    fn publish_task(
        &mut self,
        model: &Outline,
        change: &TaskChange,
        stamp: &str,
        note: &str,
    ) -> Result<Action, String> {
        let edits = model.task_edits(
            self.buffer().map_err(|e| e.to_string())?,
            change,
            stamp,
            note,
        )?;
        self.publish_org_edits(&edits).map_err(|e| e.to_string())?;
        Ok(self.message("Org task state and metadata changed"))
    }
}
fn choices(workflow: &Workflow) -> String {
    workflow
        .states
        .iter()
        .map(|s| {
            let done = if s.done { " finished" } else { "" };
            s.key.map_or_else(
                || format!("{}{done}", s.name),
                |key| format!("{}({key}){done}", s.name),
            )
        })
        .chain(std::iter::once("- unmarked".into()))
        .collect::<Vec<_>>()
        .join(", ")
}
fn select(workflow: &Workflow, draft: &str) -> Result<String, String> {
    let draft = draft.trim();
    if draft.is_empty() || draft == "-" {
        return Ok(String::new());
    }
    let state = workflow.state(draft).or_else(|| {
        (draft.chars().count() == 1)
            .then(|| {
                workflow
                    .states
                    .iter()
                    .find(|s| s.key == draft.chars().next())
            })
            .flatten()
    });
    state
        .map(|s| s.name.clone())
        .ok_or_else(|| "Unknown Org task state or selection key".into())
}

impl Session {
    fn task_timestamp(&self) -> Result<String, String> {
        if let Some(clock) = self.task_clock {
            return clock();
        }
        clock::now(&self.buffers.snapshot().settings().org.time)
    }
}
