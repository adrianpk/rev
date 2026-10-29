use super::{Action, Pending, Session, outline::Request};
use crate::{
    buffer::edit::{Cursor, Replacement},
    command::{Command, OrgCommand as C},
    coordinates::Scalar,
    input::Focus,
    org::document::{Argument, Output, Snapshot},
};
pub(super) struct Prompt {
    request: Request,
    key: Option<String>,
}
impl Session {
    pub(in crate::session) fn start_document(&mut self, request: Request) -> Result<(), String> {
        self.run_document(request, Argument::None)
    }
    fn run_document(&mut self, request: Request, argument: Argument) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let snapshot = Snapshot::new(buffer, request.origin.point)?;
        let seconds = self.buffers.snapshot().settings().org.limits.job_seconds;
        self.org
            .start_document(snapshot, request.command, argument, seconds)?;
        self.document_request = Some(request);
        self.message("Checking Org document action; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn finish_document(&mut self, result: Result<Output, String>) {
        let Some(request) = self.document_request.take() else {
            return;
        };
        if !self.org_source_current(&request)
            || self.input.focus() != Focus::Editing
            || self.pending.is_some()
        {
            self.error("Document source or view changed; source unchanged");
            return;
        }
        if let Err(error) = result.and_then(|o| self.document_output(o, request)) {
            self.error(&error);
        }
    }
    fn document_output(&mut self, output: Output, request: Request) -> Result<(), String> {
        match output {
            Output::Continue => {
                let model = self
                    .buffer()
                    .map_err(|e| e.to_string())?
                    .org_outline()
                    .cloned()
                    .ok_or("Document observation is no longer current")?;
                self.resume_document_context(model, request);
                Ok(())
            }
            Output::Message(message) => {
                self.message(&message);
                Ok(())
            }
            Output::Prompt => self.document_prompt(Prompt { request, key: None }),
            Output::Move(byte) => {
                self.link_mark();
                self.move_link_point(byte)
            }
            Output::Change { edits, cursor } => {
                self.publish_document(edits, cursor)?;
                self.message("Org document changed; source undo available");
                Ok(())
            }
        }
    }
    fn document_prompt(&mut self, prompt: Prompt) -> Result<(), String> {
        let label = match (prompt.request.command, prompt.key.is_some()) {
            (C::Emphasize, _) => "Emphasis marker (* / _ + = ~ or space): ",
            (C::InsertEntity, _) => "Org entity name: ",
            (C::EditAffiliated, false) => "Affiliated keyword (NAME/CAPTION/ATTR_BACKEND): ",
            _ => "Affiliated value: ",
        };
        self.input.argument(label, "").map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Document(Box::new(prompt)));
        self.message("Enter literal source data; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn document_argument(&mut self, prompt: Prompt, draft: &str) -> Action {
        let result = self.document_draft(prompt, draft);
        match result {
            Ok(()) => Action::Redraw,
            Err(error) => self.error(&error),
        }
    }
    fn document_draft(&mut self, mut prompt: Prompt, draft: &str) -> Result<(), String> {
        if !self.org_source_current(&prompt.request) {
            return Err("Document source or view changed; source unchanged".into());
        }
        let argument = match prompt.request.command {
            C::Emphasize => Argument::Markup(draft.into()),
            C::InsertEntity => Argument::Entity(draft.into()),
            C::EditAffiliated => {
                let Some(key) = prompt.key.take() else {
                    let key = draft.to_ascii_uppercase();
                    crate::org::document::affiliated_key(&key)?;
                    prompt.key = Some(key);
                    return self.document_prompt(prompt);
                };
                Argument::Affiliated(key, draft.into())
            }
            _ => unreachable!("document prompt"),
        };
        self.run_document(prompt.request, argument)
    }
    fn publish_document(
        &mut self,
        edits: Vec<crate::org::document::Edit>,
        cursor: Option<usize>,
    ) -> Result<(), String> {
        if edits.is_empty() {
            return Ok(());
        }
        let buffer = self
            .buffers
            .get_mut(self.windows.selected().point.stamp.buffer)
            .map_err(|e| e.to_string())?;
        let points = self.windows.points(buffer).map_err(|e| e.to_string())?;
        let edits = edits
            .into_iter()
            .map(|e| {
                buffer.byte_span(e.bytes).map(|source| Replacement {
                    source,
                    text: e.text,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let cursor = cursor.map(|at| Cursor {
            index: points.initiator,
            position: Scalar(at),
        });
        let mut prepared = buffer
            .prepare(&edits, &points.values, cursor)
            .map_err(|e| e.to_string())?;
        prepared.set_origin(self.windows.selected_id());
        self.windows
            .publish(buffer, prepared, points)
            .map_err(|e| e.to_string())?;
        buffer.deactivate_region();
        self.windows.selected_mut().reset_navigation();
        Ok(())
    }
    pub(in crate::session) fn cancel_document(&mut self, command: Command) -> Option<Action> {
        if command != Command::KeyboardQuit {
            return None;
        }
        let prompt = matches!(self.pending, Some(Pending::Document(_)));
        if self.document_request.take().is_none() && !prompt {
            return None;
        }
        self.org.cancel_document();
        self.pending = None;
        self.input.cancel();
        Some(self.message("Org document action cancelled; source unchanged"))
    }
}
