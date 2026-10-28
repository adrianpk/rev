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
    ) -> Result<ListEdit, String> {
        self.complete().map_err(str::to_owned)?;
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        if !self.lists.complete {
            return Err("Org list observation is incomplete".into());
        }
        let (byte, eof_line) = location(buffer, point)?;
        let index = self
            .lists
            .at(byte, eof_line)
            .ok_or("No editable Org list item here")?;
        if buffer.mark_active() {
            return Err("Org list structure requires an unselected item".into());
        }
        if self.malformed_between(&self.lists.lists[self.lists.items[index].list].source) {
            return Err("Org list contains an unclosed structure".into());
        }
        self.lists.edit(buffer, index, command, point)
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
            return self.insert(buffer, index, point);
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

fn location(buffer: &Buffer, point: Scalar) -> Result<(usize, bool), String> {
    let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
    let byte = coordinates.to_byte(point).map_err(|e| e.to_string())?.0;
    let column = coordinates
        .to_line(point)
        .map_err(|e| e.to_string())?
        .column;
    Ok((byte, byte == buffer.len_bytes() && column > 0))
}
