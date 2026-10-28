use crate::command::OrgCommand;
use crate::input::Focus;
use crate::view::View;
use crate::window::ViewId;
use std::sync::Arc;

pub(super) mod estimates;
mod list;
mod operation;
pub(super) mod property;
mod subtree;
pub(super) mod tags;
pub(super) mod tasks;
pub(super) mod timestamps;

use super::{Action, Session};

pub(super) struct Request {
    pub(super) view: ViewId,
    pub(super) origin: View,
    pub(super) command: OrgCommand,
}

impl Session {
    pub(super) fn org_command(&mut self, command: OrgCommand) -> Action {
        if self.input.focus() != Focus::Editing || self.tree.focused() {
            return self.message("Editing owns Org commands");
        }
        if !self.buffer().is_ok_and(|b| b.is_org()) {
            return self.message("Native Org mode is not active");
        }
        if command == OrgCommand::ShowAll {
            self.org.cancel_request();
            self.windows.selected_mut().clear_folds();
            return self.message("Org show-all");
        }
        if self.org.request.is_some() {
            return self.message("Org structural observation is pending; C-g cancels");
        }
        self.org.request = Some(Request {
            view: self.windows.selected_id(),
            origin: self.view().clone(),
            command,
        });
        self.refresh_org();
        if self.org.request.is_some() {
            return self.message("Org structural observation is pending; C-g cancels");
        }
        Action::Redraw
    }

    pub(super) fn finish_org_request(&mut self) {
        let Some(request) = &self.org.request else {
            return;
        };
        if !self.org_request_current(request) {
            self.org.request = None;
            return;
        }
        let Some(model) = self.buffer().ok().and_then(|b| b.org_outline()).cloned() else {
            return;
        };
        let request = self.org.request.take().expect("checked Org request");
        self.dispatch_org_request(model, request);
    }

    fn dispatch_org_request(&mut self, model: Arc<crate::org::Outline>, request: Request) {
        if let Err(error) = self.dispatch_outline(model, request) {
            self.error(&error);
        }
    }
    fn dispatch_outline(
        &mut self,
        model: Arc<crate::org::Outline>,
        request: Request,
    ) -> Result<(), String> {
        if request.command.timestamp() {
            return self.start_timestamp(model, request);
        }
        if request.command.estimate() {
            return self.start_estimate(model, request);
        }
        if request.command.task() {
            return self.start_task(model, request);
        }
        if request.command.property() {
            return self.start_property(model, request);
        }
        if self.tag_request(&model, request.command) {
            return self.start_tags(model, request);
        }
        self.org_operation(model, request.command)?;
        self.message(request.command.name());
        Ok(())
    }

    fn org_operation(
        &mut self,
        model: Arc<crate::org::Outline>,
        command: OrgCommand,
    ) -> Result<(), String> {
        if command.list_only()
            || command.contextual_list()
                && model.list_at(
                    self.buffer().map_err(|e| e.to_string())?,
                    self.view().point.position,
                )?
        {
            return self.edit_list(&model, command);
        }
        if let Some(subtree) = command.subtree() {
            return self.edit_subtree(&model, subtree);
        }
        self.org_outline_operation(model, command)
    }

    fn org_outline_operation(
        &mut self,
        model: Arc<crate::org::Outline>,
        command: OrgCommand,
    ) -> Result<(), String> {
        use OrgCommand::*;
        match command {
            InsertHeading | MetaReturn => self.insert_heading(model),
            NextHeading | PreviousHeading | ForwardHeading | BackwardHeading | ParentHeading => {
                self.heading_motion(&model, command)
            }
            _ => {
                let buffer = self
                    .buffers
                    .get(self.view().point.stamp.buffer)
                    .map_err(|e| e.to_string())?;
                self.windows
                    .selected_mut()
                    .org_visibility(buffer, model, command)
            }
        }
    }
    fn org_request_current(&self, request: &Request) -> bool {
        self.org_source_current(request) && self.input.focus() == Focus::Editing
    }
    pub(super) fn org_source_current(&self, request: &Request) -> bool {
        self.windows.selected_id() == request.view
            && self.view() == &request.origin
            && self
                .buffer()
                .is_ok_and(|b| b.is_org() && b.stamp() == request.origin.point.stamp)
            && !self.tree.focused()
    }
}
