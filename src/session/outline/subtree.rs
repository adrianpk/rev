use crate::buffer::edit::Error;
use crate::command::OrgCommand;
use crate::org::{Outline, SubtreeEdit};
use crate::session::Session;

impl Session {
    pub(in crate::session) fn edit_subtree(
        &mut self,
        model: &Outline,
        command: OrgCommand,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        self.view().cell(buffer).map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let plan = model.subtree_edit(buffer, command, self.view().point.position)?;
        self.publish_subtree(plan).map_err(|e| e.to_string())
    }

    fn publish_subtree(&mut self, plan: SubtreeEdit) -> Result<(), Error> {
        let buffer = self
            .buffers
            .get_mut(self.windows.selected().point.stamp.buffer)?;
        let points = self.windows.points(buffer)?;
        let mut prepared = match plan {
            SubtreeEdit::Levels(edits) => buffer.prepare(&edits, &points.values, None)?,
            SubtreeEdit::Reorder(pieces) => buffer.prepare_reorder(&pieces, &points.values)?,
        };
        prepared.retain_initiator(points.initiator)?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.windows.selected_mut().reset_navigation();
        Ok(())
    }
}
