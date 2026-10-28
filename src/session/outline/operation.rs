use std::sync::Arc;

use crate::buffer::edit::{Cursor, Error, Point};
use crate::command::OrgCommand;
use crate::coordinates::{Byte, Coordinates, LineByte, Scalar};
use crate::org::{Insertion, Outline};
use crate::session::Session;

impl Session {
    pub(in crate::session) fn heading_motion(
        &mut self,
        model: &Outline,
        command: OrgCommand,
    ) -> Result<(), String> {
        let target = self
            .heading_target(model, command)
            .map_err(|e| e.to_string())?;
        self.place_point(target).map_err(|e| e.to_string())
    }

    fn heading_target(&self, model: &Outline, command: OrgCommand) -> Result<Point, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        self.view().cell(buffer).map_err(|e| e.to_string())?;
        let (coordinates, byte) =
            line_origin(buffer, self.view().point.position).map_err(|e| e.to_string())?;
        for target in model.targets(command, byte).map_err(str::to_owned)? {
            let position = coordinates
                .from_byte(Byte(target))
                .map_err(|e| e.to_string())?;
            let row = coordinates
                .to_line(position)
                .map_err(|e| e.to_string())?
                .row;
            if visible_only(command) && self.view().project_row(buffer, row).is_none() {
                continue;
            }
            return Ok(Point {
                position,
                ..self.view().point
            });
        }
        Err("No visible Org heading in that direction".into())
    }

    pub(in crate::session) fn insert_heading(&mut self, model: Arc<Outline>) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        self.view().cell(buffer).map_err(|e| e.to_string())?;
        let insertion = model.insertion(buffer, self.view().point.position)?;
        self.publish_heading(insertion).map_err(|e| e.to_string())
    }

    fn publish_heading(&mut self, insertion: Insertion) -> Result<(), Error> {
        let buffer = self.buffers.get_mut(insertion.edit.source.stamp.buffer)?;
        let points = self.windows.points(buffer)?;
        let cursor = Cursor {
            index: points.initiator,
            position: insertion.cursor,
        };
        let mut prepared = buffer.prepare(&[insertion.edit], &points.values, Some(cursor))?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.windows.selected_mut().reset_navigation();
        Ok(())
    }
}

fn visible_only(command: OrgCommand) -> bool {
    matches!(
        command,
        OrgCommand::NextHeading | OrgCommand::PreviousHeading
    )
}

fn line_origin(
    buffer: &crate::buffer::Buffer,
    point: Scalar,
) -> Result<(Coordinates<'_>, usize), crate::view::Error> {
    let coordinates = Coordinates::new(buffer)?;
    let row = coordinates.to_line(point)?.row;
    let start = coordinates.from_line(LineByte { row, column: 0 })?;
    let byte = coordinates.to_byte(start)?.0;
    Ok((coordinates, byte))
}
