use super::super::{ListEdit, Lists, replacement, source_text};
use crate::buffer::{Buffer, edit::Replacement};
use crate::coordinates::{Coordinates, Scalar};

impl Lists {
    pub(super) fn insert(
        &self,
        buffer: &Buffer,
        index: usize,
        point: Scalar,
    ) -> Result<ListEdit, String> {
        let item = &self.items[index];
        let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let byte = coordinates.to_byte(point).map_err(|e| e.to_string())?.0;
        let above = byte <= item.content;
        let at = if above { item.source.start } else { byte };
        let prefix = self.prefix(buffer, index, above)?;
        let (text, offset) = insertion(buffer, at, &prefix, above)?;
        let source = replacement(buffer, at..at, text)?;
        let base = source.source.range.start;
        let items = &self.lists[item.list].items;
        let position = items
            .iter()
            .position(|candidate| *candidate == index)
            .expect("item owns list position");
        let mut edits = self.numbering(buffer, items, Some((position, above)))?;
        let cursor = cursor(&edits, base, offset)?;
        merge(&mut edits, source);
        Ok(ListEdit {
            edits,
            order: Vec::new(),
            cursor: Some(Scalar(cursor)),
        })
    }

    fn prefix(&self, buffer: &Buffer, index: usize, above: bool) -> Result<String, String> {
        let item = &self.items[index];
        let mut prefix = source_text(buffer, item.source.start..item.bullet.start)?;
        prefix.push_str(&self.marker(buffer, index, above)?);
        prefix.push(' ');
        if item.description {
            prefix.push_str(" :: ");
        } else if item.checkbox.is_some() {
            prefix.push_str("[ ] ");
        }
        Ok(prefix)
    }
    fn marker(&self, buffer: &Buffer, index: usize, above: bool) -> Result<String, String> {
        let item = &self.items[index];
        let marker = source_text(buffer, item.bullet.clone())?;
        if !super::super::numbering::ordered(buffer, item)? {
            return Ok(marker);
        }
        let mut next = 1u64;
        for sibling in &self.lists[item.list].items {
            if *sibling == index {
                break;
            }
            next = self.items[*sibling]
                .reset
                .unwrap_or(next)
                .checked_add(1)
                .ok_or("Org list counter is exhausted")?;
        }
        if !above {
            next = item
                .reset
                .unwrap_or(next)
                .checked_add(1)
                .ok_or("Org list counter is exhausted")?;
        }
        Ok(format!(
            "{next}{}",
            marker.chars().last().expect("list marker")
        ))
    }
}

fn insertion(
    buffer: &Buffer,
    at: usize,
    prefix: &str,
    above: bool,
) -> Result<(String, usize), String> {
    let mut text = String::new();
    if !above {
        text.push_str(buffer.newline());
    }
    text.push_str(prefix);
    let offset = text.chars().count() - if prefix.ends_with(" :: ") { 4 } else { 0 };
    if above || at == buffer.len_bytes() && source_text(buffer, at.saturating_sub(1)..at)? == "\n" {
        text.push_str(buffer.newline());
    }
    Ok((text, offset))
}

fn merge(edits: &mut Vec<Replacement>, insertion: Replacement) {
    if let Some(edit) = edits
        .iter_mut()
        .find(|edit| edit.source.range.start == insertion.source.range.start)
    {
        edit.text.insert_str(0, &insertion.text);
    } else {
        edits.push(insertion);
    }
    edits.sort_by_key(|edit| edit.source.range.start);
}

fn cursor(edits: &[Replacement], base: usize, offset: usize) -> Result<usize, String> {
    let mut cursor = base
        .checked_add(offset)
        .ok_or("Org insertion position is exhausted")?;
    for edit in edits.iter().filter(|edit| edit.source.range.start < base) {
        cursor = cursor
            .checked_add_signed(
                edit.text.chars().count() as isize - edit.source.range.len() as isize,
            )
            .ok_or("Org insertion position is exhausted")?;
    }
    Ok(cursor)
}
