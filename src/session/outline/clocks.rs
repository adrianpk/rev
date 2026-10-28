mod departure;
mod recovery;
pub(in crate::session) use departure::{Departure, Prompt};
mod batch;
mod completion;
pub(super) mod switching;
use crate::buffer::{
    Span,
    edit::{Error, Point, Replacement},
};
use crate::command::OrgCommand;
use crate::org::{ClockInsertion, Outline, time::ClockInterval};
use crate::session::{Action, Session};
use std::sync::Arc;

#[derive(Clone)]
pub(in crate::session) struct Active {
    source: Span,
    owner: Point,
    interval: ClockInterval,
    cleanup: Option<Span>,
}
impl Session {
    pub(super) fn start_clock(
        &mut self,
        model: Arc<Outline>,
        request: super::Request,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let index = model.tag_target(buffer, request.origin.point.position)?;
        if self.clock_same_task(&model, index)? {
            self.message("Org clock already runs on this task");
            return Ok(());
        }
        let policy = &self.buffers.snapshot().settings().org.time;
        let operation = self.org_clock.unwrap_or(super::timestamps::capture)(policy)?;
        let interval = ClockInterval::new(operation.clone());
        let drawer = &self.buffers.snapshot().settings().org.clock.drawer;
        let insertion = model.clock_insertion(buffer, index, (&interval.open_source(), drawer))?;
        self.switch_clock(insertion, interval, operation.instant())?;
        self.message("Org clock started");
        Ok(())
    }
    pub(super) fn publish_clock_in(
        &mut self,
        insertion: ClockInsertion,
        interval: ClockInterval,
    ) -> Result<(), String> {
        self.publish_org_edits(&[insertion.edit])
            .map_err(|e| e.to_string())?;
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        self.active_clock = Some(Active {
            source: buffer
                .span(insertion.record)
                .expect("published clock range"),
            cleanup: insertion
                .cleanup
                .map(|range| buffer.span(range).expect("published clock wrapper")),
            owner: Point {
                stamp: buffer.stamp(),
                position: insertion.owner,
            },
            interval,
        });
        Ok(())
    }
    pub(super) fn clock_command(&mut self, command: OrgCommand) -> Action {
        match self.finish_clock(command) {
            Ok(()) => self.message(command.name()),
            Err(error) => self.error(&error),
        }
    }
    pub(super) fn finish_clock(&mut self, command: OrgCommand) -> Result<(), String> {
        let active = self.active_clock.as_ref().ok_or("No active Org clock")?;
        let buffer = self
            .buffers
            .get(active.source.stamp.buffer)
            .map_err(|e| e.to_string())?;
        buffer
            .slice(&active.source)
            .map_err(|e| format!("Org clock source changed: {e}; open interval retained"))?;
        if command == OrgCommand::ClockGoto {
            let owner = active.owner;
            self.activate(owner.stamp.buffer);
            return self.place_point(owner).map_err(|e| e.to_string());
        }
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let text = self.clock_end_source(command)?;
        let active = self.active_clock.as_ref().expect("checked active clock");
        let edit = Replacement {
            source: if text.is_empty() {
                active.cleanup.as_ref().unwrap_or(&active.source).clone()
            } else {
                active.source.clone()
            },
            text,
        };
        self.publish_clock_edit(edit)?;
        self.active_clock = None;
        Ok(())
    }
    fn clock_end_source(&self, command: OrgCommand) -> Result<String, String> {
        if command == OrgCommand::ClockCancel {
            return Ok(String::new());
        }
        let active = self.active_clock.as_ref().expect("checked active clock");
        let policy = &self.buffers.snapshot().settings().org.time;
        let operation = self.org_clock.unwrap_or(super::timestamps::capture)(policy)?;
        let record = active.interval.close(operation.instant())?;
        if self.buffers.snapshot().settings().org.clock.remove_zero
            && record.duration.minutes() == 0.0
        {
            return Ok(String::new());
        }
        let buffer = self
            .buffers
            .get(active.source.stamp.buffer)
            .map_err(|e| e.to_string())?;
        Ok(format!("{}{}", record.source, buffer.newline()))
    }
    pub(super) fn publish_clock_edit(&mut self, edit: Replacement) -> Result<(), String> {
        let previous = self.view().clone();
        let foreign = previous.point.stamp.buffer != edit.source.stamp.buffer;
        if foreign {
            let buffer = self
                .buffers
                .get(edit.source.stamp.buffer)
                .map_err(|e| e.to_string())?;
            self.windows.selected_mut().rebind(buffer);
        }
        let result = self.publish_org_edits(&[edit]).map_err(|e| e.to_string());
        if foreign {
            *self.windows.selected_mut() = previous;
        }
        result
    }
}

impl Active {
    pub(in crate::session) fn buffer(&self) -> crate::buffer::BufferId {
        self.source.stamp.buffer
    }
}
