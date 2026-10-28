use crate::buffer::edit::{Cursor, Error};
use crate::command::OrgCommand;
use crate::org::{ListEdit, Outline};
use crate::session::Session;

impl Session {
    pub(in crate::session) fn edit_list(
        &mut self,
        model: &Outline,
        command: OrgCommand,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|error| error.to_string())?;
        self.view()
            .cell(buffer)
            .map_err(|error| error.to_string())?;
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let plan = model.list_edit(
            buffer,
            command,
            self.view().point.position,
            &self.buffers.snapshot().settings().org.todo,
        )?;
        self.publish_list(plan).map_err(|error| error.to_string())
    }

    fn publish_list(&mut self, plan: ListEdit) -> Result<(), Error> {
        if plan.edits.is_empty() && plan.order.is_empty() {
            return Ok(());
        }
        let buffer = self
            .buffers
            .get_mut(self.windows.selected().point.stamp.buffer)?;
        let points = self.windows.points(buffer)?;
        let mut prepared = if plan.order.is_empty() {
            buffer.prepare(
                &plan.edits,
                &points.values,
                plan.cursor.map(|position| Cursor {
                    index: points.initiator,
                    position,
                }),
            )?
        } else {
            buffer.prepare_reorder_edited(&plan.order, &plan.edits, &points.values)?
        };
        prepared.retain_initiator(points.initiator)?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.windows.selected_mut().reset_navigation();
        Ok(())
    }
}
