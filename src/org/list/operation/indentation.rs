use super::super::{ListEdit, Lists, replacement, source_text};
use crate::buffer::Buffer;
use crate::command::OrgCommand;

impl Lists {
    pub(super) fn indent(
        &self,
        buffer: &Buffer,
        index: usize,
        command: OrgCommand,
    ) -> Result<ListEdit, String> {
        use OrgCommand::*;
        let item = &self.items[index];
        let tree = matches!(command, IndentItemTree | OutdentItemTree);
        let right = matches!(command, IndentItem | IndentItemTree | MetaRight);
        let list = &self.lists[item.list];
        let position = list
            .items
            .iter()
            .position(|candidate| *candidate == index)
            .expect("list owns item");
        let children = self
            .lists
            .iter()
            .filter(|list| list.parent == Some(index))
            .map(|list| list.source.start)
            .min();
        if !right && !tree && children.is_some() {
            return Err(
                "Cannot outdent an Org item without its children; use the tree command".into(),
            );
        }
        let target = if right {
            position
                .checked_sub(1)
                .map(|p| self.items[list.items[p]].column)
                .or_else(|| tree.then_some(item.indent + 2))
                .ok_or("No previous Org item to become the parent")?
        } else {
            list.parent.map_or(0, |parent| self.items[parent].indent)
        };
        if target == item.indent {
            return Err("Org item is already at its outermost level".into());
        }
        let end = if tree {
            item.source.end
        } else {
            children.unwrap_or(item.source.end)
        };
        let edits = indentation(
            buffer,
            item.source.start..end,
            target as isize - item.indent as isize,
        )?;
        Ok(ListEdit {
            edits,
            order: Vec::new(),
            cursor: None,
        })
    }
}

fn indentation(
    buffer: &Buffer,
    range: std::ops::Range<usize>,
    delta: isize,
) -> Result<Vec<crate::buffer::edit::Replacement>, String> {
    let text = source_text(buffer, range.clone())?;
    let mut edits = Vec::new();
    let mut start = range.start;
    for line in text.split_inclusive('\n') {
        let body = line.trim_start_matches([' ', '\t']);
        let prefix = line.len() - body.len();
        if !body.trim().is_empty() {
            let column = super::super::model::columns(&line[..prefix]);
            let next = column
                .checked_add_signed(delta)
                .ok_or("Org list indentation would become negative")?;
            let mut spaces = " ".repeat(next);
            let mut end = start + prefix;
            if next == 0 && body.starts_with("* ") {
                end += 1;
                spaces.push('-');
            }
            edits.push(replacement(buffer, start..end, spaces)?);
        }
        start += line.len();
    }
    Ok(edits)
}
