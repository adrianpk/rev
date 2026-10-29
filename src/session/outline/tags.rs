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
}
impl Prompt {
    pub(in crate::session) fn agenda_append(&self, draft: &str) -> Result<String, String> {
        self.model.tag_appended(self.index, draft)
    }
}
impl Session {
    pub(super) fn tag_request(&self, model: &Outline, command: OrgCommand) -> bool {
        command.tags()
            || command == OrgCommand::CtrlCCtrlC
                && self
                    .buffer()
                    .is_ok_and(|b| model.tag_header_at(b, self.view().point.position))
    }
    pub(super) fn start_tags(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let index = model.tag_target(buffer, self.view().point.position)?;
        if request.command == OrgCommand::InspectTags {
            let text = model.tag_inspection(
                buffer,
                index,
                self.buffers.snapshot().settings().org.tags.inherit,
            )?;
            self.message(&text);
            return Ok(());
        }
        if request.command == OrgCommand::MatchTags {
            self.input
                .argument("Org tag/property match: ", "")
                .map_err(|e| e.to_string())?;
            self.pending = Some(Pending::Tags(Box::new(Prompt {
                request,
                model,
                index,
            })));
            self.message("Match the current heading; C-g cancels");
            return Ok(());
        }
        self.begin_tag_edit(model, request, index)
    }
    fn begin_tag_edit(
        &mut self,
        model: Arc<Outline>,
        request: Request,
        index: usize,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let label = if request.command == OrgCommand::RemoveTag {
            "Remove local Org tag: "
        } else {
            "Local Org tags: "
        };
        let feedback = model.tag_inspection(
            buffer,
            index,
            self.buffers.snapshot().settings().org.tags.inherit,
        )?;
        self.input.argument(label, "").map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Tags(Box::new(Prompt {
            request,
            model,
            index,
        })));
        self.message(&feedback);
        Ok(())
    }
    pub(in crate::session) fn tag_argument(&mut self, prompt: Prompt, draft: &str) -> Action {
        if !self.org_request_current(&prompt.request) {
            return self.message("Org tag source or view changed; request cancelled");
        }
        if prompt.request.command == OrgCommand::MatchTags {
            return self.match_tag_argument(&prompt, draft);
        }
        let result = self.buffer().map_err(|e| e.to_string()).and_then(|buffer| {
            prompt.model.tag_edit(
                buffer,
                prompt.index,
                draft,
                prompt.request.command == OrgCommand::RemoveTag,
            )
        });
        match result.and_then(|edits| self.publish_org_edits(&edits).map_err(|e| e.to_string())) {
            Ok(()) => self.message("Local Org tags changed; inherited owners retained"),
            Err(error) => self.error(&error),
        }
    }
    fn match_tag_argument(&mut self, prompt: &Prompt, draft: &str) -> Action {
        let result = crate::org::MatchQuery::parse(draft).and_then(|query| {
            let buffer = self.buffer().map_err(|e| e.to_string())?;
            prompt.model.match_tags(
                buffer,
                prompt.index,
                &query,
                &self.buffers.snapshot().settings().org,
            )
        });
        match result {
            Ok(true) => self.message("Org match: yes"),
            Ok(false) => self.message("Org match: no"),
            Err(error) => self.error(&error),
        }
    }
    pub(in crate::session) fn complete_tag(&mut self) -> Action {
        let (Some(Pending::Tags(prompt)), Some(draft)) = (&self.pending, self.input.prompt())
        else {
            return self.message("No Org tag argument");
        };
        if prompt.request.command == OrgCommand::MatchTags {
            return self.message("Enter a bounded tag/property expression; C-g cancels");
        }
        // Completion owns only the prompt; an accepted draft rechecks the source.
        let result = prompt.model.tag_completion(draft.text(), draft.cursor());
        match result {
            Ok((text, feedback)) => match self.input.complete(&text) {
                Ok(()) => self.message(&feedback),
                Err(error) => self.error(&error.to_string()),
            },
            Err(error) => self.error(&error),
        }
    }
}
