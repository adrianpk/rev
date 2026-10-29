use std::collections::HashMap;
use std::ops::Range;

use super::{Item, List, Lists};
use crate::org::{Cancellation, Coverage, Structure, StructureKind};

#[derive(Default)]
struct Owners {
    lists: HashMap<usize, usize>,
    items: HashMap<usize, usize>,
}

impl Lists {
    pub(in crate::org) fn observe(
        structure: &Structure,
        text: &ropey::Rope,
        cancel: &Cancellation,
    ) -> Self {
        let mut model = Self {
            complete: structure.coverage() == Coverage::Complete,
            ..Self::default()
        };
        let mut owners = Owners::default();
        for (id, node) in structure.raw_nodes().enumerate() {
            if cancel.is_cancelled()
                || model.items.len() + model.lists.len() + model.headings.len() == 8192
                || model.cookie_count > 4096
            {
                model.complete = false;
                break;
            }
            model.collect(node, text, &mut owners, id);
        }
        model.complete &= model.cookie_count <= 4096;
        model.editable.sort_by_key(|range| range.start);
        model
    }

    fn collect(
        &mut self,
        node: &crate::org::structure::Raw,
        text: &ropey::Rope,
        owners: &mut Owners,
        id: usize,
    ) {
        match node.kind {
            StructureKind::List { .. } => {
                owners.lists.insert(id, self.lists.len());
                self.lists.push(List {
                    source: node.source.clone(),
                    parent: node.parent.and_then(|p| owners.items.get(&p).copied()),
                    items: Vec::new(),
                    radio: false,
                });
            }
            StructureKind::Item { indent } => {
                let list = node
                    .parent
                    .and_then(|p| owners.lists.get(&p))
                    .copied()
                    .expect("parsed list owns item");
                let mut item = read(text, node, list, indent);
                item.cookies = super::progress::cookies(
                    text,
                    item.header.clone(),
                    4096 - self.cookie_count.min(4096),
                );
                self.cookie_count += item.cookies.len();
                owners.items.insert(id, self.items.len());
                self.lists[list].items.push(self.items.len());
                self.editable.push(item.header.clone());
                self.items.push(item);
            }
            StructureKind::Paragraph => self.editable.push(node.source.clone()),
            _ => self.progress_node(node, text, &owners.lists),
        }
    }

    pub(super) fn at(&self, byte: usize, eof_line: bool) -> Option<usize> {
        if !self
            .editable
            .iter()
            .any(|range| range.contains(&byte) || eof_line && range.end == byte)
        {
            return None;
        }
        self.items
            .iter()
            .rposition(|item| item.source.contains(&byte) || eof_line && item.source.end == byte)
    }
}

pub(super) fn read(
    text: &ropey::Rope,
    node: &crate::org::structure::Raw,
    list: usize,
    indent: usize,
) -> Item {
    let row = text.byte_to_line(node.source.start);
    let line = text.line(row).to_string();
    let line = line.trim_end_matches(['\r', '\n']);
    let start = node.source.start;
    let first = line.len() - line.trim_start_matches([' ', '\t']).len();
    let end = line[first..]
        .find([' ', '\t'])
        .map_or(line.len(), |n| first + n);
    let mut content = node.content.start;
    let reset = reset(line, start, &mut content);
    let checkbox = checkbox(line, start, &mut content);
    Item {
        source: node.source.clone(),
        header: start..start + line.len(),
        bullet: start + first..start + end,
        content,
        indent,
        column: columns(&line[..node.content.start - start]),
        list,
        reset,
        description: line[content - start..].contains(" :: "),
        checkbox,
        cookies: Vec::new(),
    }
}

fn reset(line: &str, start: usize, content: &mut usize) -> Option<String> {
    let tail = line.get(*content - start..)?.strip_prefix("[@")?;
    let end = tail.find(']')?;
    let value = &tail[..end];
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let value = value.trim_start_matches('0');
    let value = if value.is_empty() { "0" } else { value }.to_owned();
    *content += 3 + end;
    skip_space(line, start, content);
    Some(value)
}

fn checkbox(line: &str, start: usize, content: &mut usize) -> Option<(Range<usize>, u8)> {
    let tail = line.as_bytes().get(*content - start..)?;
    if tail.len() < 3 || tail[0] != b'[' || tail[2] != b']' || !b" X-".contains(&tail[1]) {
        return None;
    }
    let range = *content..*content + 3;
    *content += 3;
    skip_space(line, start, content);
    Some((range, tail[1]))
}

fn skip_space(line: &str, start: usize, content: &mut usize) {
    *content += line[*content - start..].len()
        - line[*content - start..]
            .trim_start_matches([' ', '\t'])
            .len();
}

pub(super) fn columns(prefix: &str) -> usize {
    prefix.chars().fold(0, |column, character| {
        if character == '\t' {
            column + 8 - column % 8
        } else {
            column + 1
        }
    })
}
