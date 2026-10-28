use super::*;
use crate::{
    buffer::edit::{Cursor, Error, Replacement},
    command::OrgCommand,
    org::{ListEdit, Outline},
};
impl Session {
    pub(in crate::session) fn timer_list_request(
        &self,
        model: &Outline,
        command: OrgCommand,
    ) -> Result<bool, String> {
        if command != OrgCommand::MetaReturn {
            return Ok(false);
        }
        model.timed_list_at(
            self.buffer().map_err(|e| e.to_string())?,
            self.view().point.position,
        )
    }
    pub(in crate::session) fn insert_timer(
        &mut self,
        model: &Outline,
        command: OrgCommand,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let (next, value) = self.timer_insertion_value()?;
        let plan = if command == OrgCommand::TimerInsert {
            self.timer_text(model, &value)?
        } else {
            model.timer_item_edit(
                buffer,
                self.view().point.position,
                (&value, &self.buffers.snapshot().settings().org.todo),
            )?
        };
        self.publish_timer_source(plan).map_err(|e| e.to_string())?;
        self.timer = next;
        self.timer_notice = false;
        self.message("Org timer inserted");
        Ok(())
    }
    fn timer_insertion_value(&self) -> Result<(crate::org::time::Timer, String), String> {
        let now = self.timer_now();
        let mut next = self.timer.clone();
        if next.mode().is_none() {
            next.start(now)?;
        }
        let value = next
            .reading(now)?
            .ok_or("No Org timer is running")?
            .format();
        Ok((next, value))
    }
    fn timer_text(&self, model: &Outline, value: &str) -> Result<ListEdit, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let point = self.view().point.position;
        if !model.timestamp_context(buffer, point)? {
            return Err("Timer insertion requires ordinary Org source".into());
        }
        let text = format!("{value} ");
        let cursor = crate::coordinates::Scalar(point.0 + text.chars().count());
        Ok(ListEdit {
            edits: vec![Replacement {
                source: buffer.span(point.0..point.0).map_err(|e| e.to_string())?,
                text,
            }],
            order: Vec::new(),
            cursor: Some(cursor),
        })
    }
    fn publish_timer_source(&mut self, plan: ListEdit) -> Result<(), Error> {
        let buffer = self.buffers.get_mut(self.view().point.stamp.buffer)?;
        let points = self.windows.points(buffer)?;
        let mut prepared = buffer.prepare(
            &plan.edits,
            &points.values,
            plan.cursor.map(|position| Cursor {
                index: points.initiator,
                position,
            }),
        )?;
        prepared.retain_initiator(points.initiator)?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.windows.selected_mut().reset_navigation();
        Ok(())
    }
}
