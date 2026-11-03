use crate::command::OrgCommand;

use super::super::{Action, Session};

impl Session {
    pub(super) fn heading_presentation(&mut self, command: OrgCommand) -> Option<Action> {
        let view = self.windows.selected_mut();
        match command {
            OrgCommand::PrettyHeadings => view.toggle_heading_presentation(),
            OrgCommand::ResetHeadingPresentation => view.reset_heading_presentation(),
            _ => return None,
        }
        if view.pretty_headings() {
            let buffer = view.point.stamp.buffer;
            if let Ok(buffer) = self.buffers.get_mut(buffer) {
                buffer.request_org_outline();
            }
        }
        Some(self.message(if self.view().pretty_headings() {
            "Org headings: pretty"
        } else {
            "Org headings: source"
        }))
    }
}
