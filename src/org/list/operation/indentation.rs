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
        let whole = tree && position == 0 && list.parent.is_none();
        let target = self.indent_target(index, right, whole)?;
        if target == item.indent {
            return Err("Org item is already at its outermost level".into());
        }
        let end = self.item_end(index, whole, tree);
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
    fn item_end(&self, index: usize, whole: bool, tree: bool) -> usize {
        let item = &self.items[index];
        if whole {
            return self.lists[item.list].source.end;
        }
        if tree {
            return item.source.end;
        }
        self.lists
            .iter()
            .filter(|list| list.parent == Some(index))
            .map(|list| list.source.start)
            .min()
            .unwrap_or(item.source.end)
    }

    fn indent_target(&self, index: usize, right: bool, whole: bool) -> Result<usize, String> {
        let item = &self.items[index];
        if whole {
            return if right {
                Ok(item.indent + 1)
            } else {
                item.indent
                    .checked_sub(1)
                    .ok_or_else(|| "Org item is already at its outermost level".into())
            };
        }
        let list = &self.lists[item.list];
        if !right {
            return list
                .parent
                .map(|parent| self.items[parent].indent)
                .ok_or_else(|| {
                    "Cannot outdent top-level Org items; use the whole-list tree command".into()
                });
        }
        let position = list
            .items
            .iter()
            .position(|candidate| *candidate == index)
            .expect("list owns item");
        position
            .checked_sub(1)
            .map(|previous| self.items[list.items[previous]].column)
            .ok_or_else(|| "No previous Org item to become the parent".into())
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
