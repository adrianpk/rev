use super::super::{ListEdit, Lists};
use crate::buffer::{Buffer, Span};
use crate::command::OrgCommand;

impl Lists {
    pub(super) fn move_item(
        &self,
        buffer: &Buffer,
        index: usize,
        command: OrgCommand,
    ) -> Result<ListEdit, String> {
        let item = &self.items[index];
        let list = &self.lists[item.list];
        let mut items = list.items.clone();
        let position = items
            .iter()
            .position(|candidate| *candidate == index)
            .expect("list owns item");
        let next = if matches!(command, OrgCommand::MoveItemDown | OrgCommand::MetaDown) {
            position.checked_add(1)
        } else {
            position.checked_sub(1)
        };
        let other = next
            .filter(|next| *next < items.len())
            .ok_or("No Org list sibling in that direction")?;
        items.swap(position, other);
        let left = &self.items[list.items[position.min(other)]];
        let right = &self.items[list.items[position.max(other)]];
        let mut order = Vec::new();
        append(buffer, &mut order, list.source.start..left.source.start)?;
        let a = buffer
            .byte_span(left.source.start..right.source.start)
            .map_err(|e| e.to_string())?;
        let b = buffer
            .byte_span(right.source.clone())
            .map_err(|e| e.to_string())?;
        order.extend(
            crate::org::visibility::movement_pieces(buffer, &a, &b).map_err(|e| e.to_string())?,
        );
        append(buffer, &mut order, right.source.end..list.source.end)?;
        Ok(ListEdit {
            edits: self.numbering(buffer, &items, None)?,
            order,
            cursor: None,
        })
    }
}

fn append(
    buffer: &Buffer,
    spans: &mut Vec<Span>,
    bytes: std::ops::Range<usize>,
) -> Result<(), String> {
    if !bytes.is_empty() {
        spans.push(buffer.byte_span(bytes).map_err(|e| e.to_string())?);
    }
    Ok(())
}
