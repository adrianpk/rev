use crate::command::OrgCommand;
use crate::input::Focus;
use crate::view::View;
use crate::window::ViewId;

use super::{Action, Session};

pub(super) struct Request {
    pub(super) view: ViewId,
    pub(super) origin: View,
    pub(super) command: OrgCommand,
}

impl Session {
    pub(super) fn visibility_command(&mut self, command: OrgCommand) -> Action {
        if self.input.focus() != Focus::Editing || self.tree.focused() {
            return self.message("Editing owns Org visibility commands");
        }
        if !self.buffer().is_ok_and(|b| b.is_org()) {
            return self.message("Native Org mode is not active");
        }
        if command == OrgCommand::ShowAll {
            self.org.cancel_visibility();
            self.windows.selected_mut().clear_folds();
            return self.message("Org show-all");
        }
        if self.org.request.is_some() {
            return self.message("Org visibility observation is pending; C-g cancels");
        }
        self.org.request = Some(Request {
            view: self.windows.selected_id(),
            origin: self.view().clone(),
            command,
        });
        self.refresh_org();
        if self.org.request.is_some() {
            return self.message("Org visibility observation is pending; C-g cancels");
        }
        Action::Redraw
    }

    pub(super) fn finish_visibility(&mut self) {
        let Some(request) = &self.org.request else {
            return;
        };
        if !self.visibility_current(request) {
            self.org.request = None;
            return;
        }
        let Some(model) = self.buffer().ok().and_then(|b| b.org_outline()).cloned() else {
            return;
        };
        let request = self.org.request.take().expect("checked visibility request");
        let buffer = match self.buffers.get(request.origin.point.stamp.buffer) {
            Ok(b) => b,
            Err(_) => return,
        };
        let result = self
            .windows
            .selected_mut()
            .org_visibility(buffer, model, request.command);
        match result {
            Ok(()) => {
                self.message(request.command.name());
            }
            Err(error) => {
                self.error(&error);
            }
        }
    }
    fn visibility_current(&self, request: &Request) -> bool {
        self.windows.selected_id() == request.view
            && self.view() == &request.origin
            && self
                .buffer()
                .is_ok_and(|b| b.is_org() && b.stamp() == request.origin.point.stamp)
            && self.input.focus() == Focus::Editing
            && !self.tree.focused()
    }
}
