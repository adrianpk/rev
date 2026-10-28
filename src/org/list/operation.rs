mod indentation;
mod insertion;
mod movement;

use crate::buffer::Buffer;
use crate::command::OrgCommand;
use crate::coordinates::{Coordinates, Scalar};
use crate::org::Outline;

use super::{ListEdit, Lists};

impl Outline {
    pub(crate) fn list_at(&self, buffer: &Buffer, point: Scalar) -> Result<bool, String> {
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        let (byte, eof_line) = location(buffer, point)?;
        Ok(self.lists.at(byte, eof_line).is_some())
    }

    pub(crate) fn list_edit(
        &self,
        buffer: &Buffer,
        command: OrgCommand,
        point: Scalar,
        policy: &crate::config::Todo,
    ) -> Result<ListEdit, String> {
        let (byte, eof_line) = self.admit_list(buffer, point)?;
        let workflow = self.task_workflow(policy)?;
        if command.checkbox() {
            let selected = self.lists.checkbox_selection(buffer, command, point)?;
            for index in selected {
                self.admit_list_scope(self.lists.items[index].source.start)?;
            }
            return self.lists.checkbox_edit(buffer, command, point, &workflow);
        }
        let index = self
            .lists
            .at(byte, eof_line)
            .ok_or("No editable Org list item here")?;
        if buffer.mark_active() {
            return Err("Org list structure requires an unselected item".into());
        }
        let plan = self.lists.edit(buffer, index, command, point)?;
        self.lists.structural_progress(
            buffer,
            super::Target {
                index,
                command,
                point,
            },
            plan,
            &workflow,
        )
    }
    pub(in crate::org::list) fn admit_list(
        &self,
        buffer: &Buffer,
        point: Scalar,
    ) -> Result<(usize, bool), String> {
        self.complete().map_err(str::to_owned)?;
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        if !self.lists.complete {
            return Err("Org list observation is incomplete".into());
        }
        let location = location(buffer, point)?;
        self.admit_list_scope(location.0)?;
        Ok(location)
    }

    fn admit_list_scope(&self, byte: usize) -> Result<(), String> {
        let heading = self.lists.heading_at(byte);
        let start = heading.map_or(0, |index| self.lists.headings[index].source.start);
        let end = self
            .lists
            .headings
            .get(heading.map_or(0, |index| index + 1))
            .map_or(usize::MAX, |heading| heading.source.start);
        if self.malformed_between(&(start..end)) {
            return Err("Org list entry contains an unclosed structure".into());
        }
        Ok(())
    }
}

impl Lists {
    fn edit(
        &self,
        buffer: &Buffer,
        index: usize,
        command: OrgCommand,
        point: Scalar,
    ) -> Result<ListEdit, String> {
        if command.list_insertion() {
            return self.insert_with(buffer, index, point, false);
        }
        if command.list_indentation() {
            return self.indent(buffer, index, command);
        }
        if command.list_movement() {
            return self.move_item(buffer, index, command);
        }
        if command == OrgCommand::RenumberList {
            return Ok(ListEdit {
                edits: self.numbering(buffer, &self.lists[self.items[index].list].items, None)?,
                order: Vec::new(),
                cursor: None,
            });
        }
        Err("Org list operation is unavailable".into())
    }
}

pub(super) fn location(buffer: &Buffer, point: Scalar) -> Result<(usize, bool), String> {
    let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
    let byte = coordinates.to_byte(point).map_err(|e| e.to_string())?.0;
    let column = coordinates
        .to_line(point)
        .map_err(|e| e.to_string())?
        .column;
    Ok((byte, byte == buffer.len_bytes() && column > 0))
}
