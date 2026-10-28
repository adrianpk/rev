mod effort;
use super::Request;
use crate::buffer::edit::Error;
use crate::command::OrgCommand;
use crate::org::Outline;
use crate::session::{Action, Pending, Session};
use std::sync::Arc;

pub(in crate::session) struct Prompt {
    request: Request,
    model: Arc<Outline>,
    index: usize,
    choices: Vec<String>,
}
impl Session {
    pub(super) fn start_estimate(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let index = model.tag_target(buffer, self.view().point.position)?;
        let prompt = Prompt {
            request,
            model,
            index,
            choices: Vec::new(),
        };
        if prompt.request.command.effort() {
            return self.start_effort(prompt);
        }
        self.start_priority(prompt)
    }
    fn start_priority(&mut self, prompt: Prompt) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let policy = &self.buffers.snapshot().settings().org;
        let priorities = prompt.model.priority_policy(policy)?;
        let current = prompt.model.priority_value(prompt.index, policy)?;
        if prompt.request.command == OrgCommand::InspectPriority {
            self.message(&format!(
                "Org priority: {} ({})",
                char::from(current.unwrap_or(priorities.default)),
                if current.is_some() {
                    "explicit"
                } else {
                    "default"
                }
            ));
            return Ok(());
        }
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        if prompt.request.command == OrgCommand::Priority {
            let label = format!(
                "Priority {}..{}; empty/space clears; C-g cancels",
                char::from(priorities.highest),
                char::from(priorities.lowest)
            );
            self.input
                .argument("Org priority: ", "")
                .map_err(|e| e.to_string())?;
            self.pending = Some(Pending::Estimate(Box::new(prompt)));
            self.message(&label);
            return Ok(());
        }
        let next = priorities.step(current, prompt.request.command == OrgCommand::PriorityUp);
        self.publish_priority(&prompt, next).map(|_| ())
    }
    fn publish_priority(&mut self, prompt: &Prompt, next: Option<u8>) -> Result<Action, String> {
        let edits = prompt.model.priority_edit(
            self.buffer().map_err(|e| e.to_string())?,
            prompt.index,
            next,
            &self.buffers.snapshot().settings().org,
        )?;
        self.publish_org_edits(&edits).map_err(|e| e.to_string())?;
        Ok(self.message("Org priority changed"))
    }
    pub(in crate::session) fn estimate_argument(&mut self, prompt: Prompt, draft: &str) -> Action {
        if !self.org_request_current(&prompt.request) {
            return self.message("Org estimate source or view changed; request cancelled");
        }
        let result = if prompt.request.command.effort() {
            self.effort_argument(&prompt, draft)
        } else {
            prompt
                .model
                .priority_policy(&self.buffers.snapshot().settings().org)
                .and_then(|p| p.admit(draft))
                .and_then(|next| self.publish_priority(&prompt, next))
        };
        match result {
            Ok(action) => action,
            Err(error) => self.error(&error),
        }
    }
}

impl Session {
    pub(in crate::session) fn complete_estimate(&mut self) -> Action {
        let (Some(Pending::Estimate(prompt)), Some(draft)) = (&self.pending, self.input.prompt())
        else {
            return self.message("No Org estimate argument");
        };
        let matches: Vec<_> = prompt
            .choices
            .iter()
            .filter(|s| s.starts_with(draft.text()))
            .collect();
        if matches.len() == 1 {
            let value = matches[0].clone();
            return match self.input.complete(&value) {
                Ok(()) => self.message("Org effort choice completed"),
                Err(e) => self.error(&e.to_string()),
            };
        }
        self.message("Enter an admitted priority/duration; Tab completes a unique EFFORT_ALL choice; C-g cancels")
    }
}
