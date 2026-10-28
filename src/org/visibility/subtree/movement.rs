use crate::buffer::{Buffer, Span};
use crate::command::OrgCommand;
use crate::coordinates::Byte;
use crate::org::Source;

use super::{Group, Outline};

impl Outline {
    pub(super) fn move_group(
        &self,
        buffer: &Buffer,
        group: &Group,
        command: OrgCommand,
    ) -> Result<Vec<Span>, String> {
        let down = command == OrgCommand::MoveSubtreeDown;
        let index = if down {
            *group.roots.last().ok_or("No Org subtree")?
        } else {
            group.roots[0]
        };
        let other = self
            .sibling(index, down)
            .ok_or("No Org sibling in that direction")?;
        let h = &self.headings[other];
        let peer = self.group(h.start..h.end).map_err(str::to_owned)?;
        self.admit_group(&peer).map_err(str::to_owned)?;
        let (start, middle, end) = if down {
            (group.bytes.start, group.bytes.end, h.end)
        } else {
            (h.start, group.bytes.start, group.bytes.end)
        };
        let left = Source::new(buffer)
            .range(Byte(start)..Byte(middle))
            .map_err(|e| e.to_string())?;
        let right = Source::new(buffer)
            .range(Byte(middle)..Byte(end))
            .map_err(|e| e.to_string())?;
        pieces(buffer, left.span(), right.span()).map_err(|e| e.to_string())
    }
}

pub(in crate::org) fn pieces(
    buffer: &Buffer,
    left: &Span,
    right: &Span,
) -> Result<Vec<Span>, crate::buffer::Error> {
    let tail = buffer.slice(right)?;
    if right.range.end != buffer.len_chars()
        || tail.get_char(tail.len_chars().saturating_sub(1)) == Some('\n')
    {
        return Ok(vec![right.clone(), left.clone()]);
    }
    let text = buffer.slice(left)?;
    let mut source = text.chars_at(text.len_chars()).reversed().peekable();
    let mut split = left.range.end;
    while source.next() == Some('\n') {
        split -= 1;
        if source.peek() == Some(&'\r') {
            source.next();
            split -= 1;
        }
    }
    Ok(vec![
        right.clone(),
        buffer.span(split..left.range.end)?,
        buffer.span(left.range.start..split)?,
    ])
}
