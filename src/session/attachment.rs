//! One explicit attachment interaction; file effects outlive cancelled prompts.
mod prompt;
mod results;
use super::{Action, Pending, Session, outline::Request};
use crate::{
    command::{Command, OrgCommand as C},
    org::{
        Outline,
        attachment::{Context, Effect, FileOutcome, Listing, Method, Plan},
    },
};
pub use results::{AttachmentResult, HeadingStatus};
use std::{collections::VecDeque, sync::Arc};
pub(super) struct State {
    request: Request,
    model: Arc<Outline>,
    context: Context,
    stage: Stage,
    cancelled: bool,
}
enum Stage {
    Dispatcher,
    Source(Method),
    Directory,
    Listing(Option<bool>),
    Choose(Listing, bool),
    Review(Box<Plan>),
    Running,
}
impl Session {
    pub(in crate::session) fn start_attachment(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        if self.capture.is_some()
            || self.refile.is_some()
            || self.links.request.is_some()
            || self.attachment.is_some()
        {
            return Err("Finish or cancel the active source interaction before attachments".into());
        }
        let context = Context::new(
            &model,
            self.buffer().map_err(|e| e.to_string())?,
            request.origin.point.position,
            &self.buffers.snapshot().settings().org,
        )?;
        let command = request.command;
        self.attachment = Some(State {
            request,
            model,
            context,
            stage: Stage::Dispatcher,
            cancelled: false,
        });
        let result = self.attachment_command(command);
        if result.is_err() {
            self.attachment = None;
        }
        result
    }
    fn attachment_command(&mut self, command: C) -> Result<(), String> {
        match command {
            C::Attach => self.attachment_prompt(
                Stage::Dispatcher,
                "Attachment [a/c/m/l/o/O/f/s]: ",
                "Copy/move/hard-link selected files; o/O opens; f lists; s sets DIR",
            ),
            C::AttachCopy | C::AttachFile => self.attachment_source(Method::Copy),
            C::AttachMove => self.attachment_source(Method::Move),
            C::AttachLink => self.attachment_source(Method::Link),
            C::AttachOpen => self.attachment_list(Some(false)),
            C::AttachOpenNative => self.attachment_list(Some(true)),
            C::AttachList => self.attachment_list(None),
            C::AttachDirectory => self.attachment_prompt(
                Stage::Directory,
                "Attachment DIR: ",
                "Set a literal directory; no folder is created",
            ),
            _ => unreachable!("attachment command"),
        }
    }
    fn attachment_source(&mut self, method: Method) -> Result<(), String> {
        if self.buffer().map_err(|e| e.to_string())?.is_readonly() {
            return Err("Attachment heading is read-only".into());
        }
        self.attachment_prompt(
            Stage::Source(method),
            "Attach file: ",
            "Select an existing local file; confirmation follows; C-g cancels",
        )
    }
    fn attachment_list(&mut self, open: Option<bool>) -> Result<(), String> {
        let state = self
            .attachment
            .as_mut()
            .ok_or("No attachment interaction")?;
        let context = state.context.clone();
        state.stage = Stage::Listing(open);
        self.org
            .start_attachment_listing(state.request.origin.point.stamp, context)?;
        self.message("Listing checked attachments; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn finish_attachment_listing(
        &mut self,
        result: Result<Listing, String>,
    ) {
        let Some(state) = self.attachment.as_ref() else {
            return;
        };
        if !self.org_source_current(&state.request) || state.cancelled {
            self.attachment = None;
            return;
        }
        if let Err(e) = result.and_then(|listing| self.attachment_listed(listing)) {
            self.reject_attachment(&e);
        }
    }
    fn attachment_listed(&mut self, listing: Listing) -> Result<(), String> {
        let state = self
            .attachment
            .as_ref()
            .ok_or("No attachment interaction")?;
        let Stage::Listing(open) = state.stage else {
            return Err("Unexpected attachment listing".into());
        };
        let labels = listing
            .names
            .iter()
            .enumerate()
            .take(8)
            .map(|(n, s)| format!("{}: {s}", n + 1))
            .collect::<Vec<_>>()
            .join("; ");
        let feedback = format!(
            "{} attachments at {}: {labels}",
            listing.names.len(),
            listing.root.display()
        );
        match open {
            Some(native) if !listing.names.is_empty() => self.attachment_prompt(
                Stage::Choose(listing, native),
                "Open attachment [number/name]: ",
                &feedback,
            ),
            _ => {
                self.attachment = None;
                self.message(&feedback);
                Ok(())
            }
        }
    }
    pub(in crate::session) fn finish_attachment_plan(&mut self, result: Result<Plan, String>) {
        let Some(state) = self.attachment.as_ref() else {
            return;
        };
        if !self.org_source_current(&state.request) || state.cancelled {
            self.attachment = None;
            return;
        }
        let result = result.and_then(|plan| {
            self.attachment_edits(&plan)?;
            let feedback = format!(
                "{:?} {} to {}; {}",
                plan.method,
                plan.source.display(),
                plan.destination.display(),
                if plan.replacing() {
                    "replace existing file?"
                } else {
                    "create new attachment?"
                }
            );
            self.attachment_prompt(
                Stage::Review(Box::new(plan)),
                "Confirm attachment yes/no: ",
                &feedback,
            )
        });
        if let Err(e) = result {
            self.reject_attachment(&e);
        }
    }
    fn attachment_edits(
        &self,
        plan: &Plan,
    ) -> Result<Vec<crate::buffer::edit::Replacement>, String> {
        self.attachment_move_admission(plan)?;
        let edits = self.attachment_metadata(plan.id.as_deref())?;
        let _ = self
            .buffer()
            .map_err(|e| e.to_string())?
            .prepare(&edits, &[], None)
            .map_err(|e| e.to_string())?;
        Ok(edits)
    }
    fn attachment_move_admission(&self, plan: &Plan) -> Result<(), String> {
        if plan.method == Method::Move
            && self
                .buffers
                .iter()
                .any(|b| b.file().is_some_and(|f| f.path() == plan.source))
        {
            return Err("Save and close the selected input buffer before moving its file".into());
        }
        Ok(())
    }
    fn attachment_origin(&self) -> Result<&State, String> {
        let state = self.attachment.as_ref().ok_or("No attachment source")?;
        if state.cancelled || !self.org_source_current(&state.request) {
            return Err("Attachment source or view changed/cancelled".into());
        }
        if self.buffer().map_err(|e| e.to_string())?.is_readonly() {
            return Err("Attachment source is read-only".into());
        }
        Ok(state)
    }
    fn attachment_metadata(
        &self,
        id: Option<&str>,
    ) -> Result<Vec<crate::buffer::edit::Replacement>, String> {
        let state = self.attachment_origin()?;
        let b = self.buffer().map_err(|e| e.to_string())?;
        let heading = state
            .model
            .tag_target(b, state.request.origin.point.position)?;
        let mut edits = Vec::new();
        if let Some(id) = id {
            let entry = state.model.heading_property_entry(b, heading)?;
            edits.extend(state.model.property_edit(b, entry, "ID", Some(id))?);
        }
        let tags = state.model.tag_appended(heading, "ATTACH")?;
        edits.extend(state.model.tag_edit(b, heading, &tags, false)?);
        edits.sort_by_key(|edit| edit.source.range.start);
        Ok(edits)
    }
    pub(in crate::session) fn cancel_attachment(&mut self, command: Command) -> Option<Action> {
        if command != Command::KeyboardQuit || self.attachment.is_none() {
            return None;
        }
        let state = self
            .attachment
            .as_mut()
            .expect("owned attachment interaction");
        let running = matches!(state.stage, Stage::Running | Stage::Listing(_));
        state.cancelled = true;
        self.org.cancel();
        self.pending = None;
        self.input.cancel();
        if !running {
            self.attachment = None;
        }
        Some(
            self.message(
                "Attachment cancelled; completed file effects remain in attachment results",
            ),
        )
    }
    pub(in crate::session) fn attachment_source_changed(&self) -> bool {
        self.attachment
            .as_ref()
            .is_some_and(|s| s.cancelled || !self.org_source_current(&s.request))
    }
    fn reject_attachment(&mut self, error: &str) -> Action {
        self.attachment = None;
        self.pending = None;
        self.input.cancel();
        self.error(&format!("Attachment rejected: {error}"))
    }
}

#[cfg(test)]
mod tests;
