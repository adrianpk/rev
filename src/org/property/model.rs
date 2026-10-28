use super::*;
use crate::org::{Coverage, StructureKind};
use ropey::Rope;
use std::collections::BTreeMap;

impl Properties {
    pub(in crate::org) fn observe(
        structure: &Structure,
        text: &Rope,
        cancel: &Cancellation,
    ) -> Self {
        let mut result = Self {
            complete: structure.coverage() == Coverage::Complete,
            ..Self::default()
        };
        result.entries.push(Entry {
            start: 0,
            end: text.len_bytes(),
            anchor: 0,
            parent: None,
            invalid: false,
            drawer: None,
            records: Vec::new(),
        });
        let mut owners = BTreeMap::new();
        let mut root_prefix = true;
        for (id, node) in structure.raw_nodes().enumerate() {
            if cancel.is_cancelled() || result.records.len() >= 8192 || result.entries.len() >= 4096
            {
                result.complete = false;
                break;
            }
            let owner = node
                .parent
                .and_then(|p| owners.get(&p).copied())
                .unwrap_or(0);
            let direct = node.parent.is_none_or(|p| owners.contains_key(&p));
            let owner = if direct {
                result.root_anchor(node, &mut root_prefix);
                result.metadata(node, text, owner);
                result.collect(node, text, owner)
            } else {
                owner
            };
            if matches!(
                node.kind,
                StructureKind::Heading { .. }
                    | StructureKind::Section
                    | StructureKind::PropertyDrawer
            ) {
                owners.insert(id, owner);
            }
        }
        if result.entries.len() > 1 {
            result.entries[0].end = result.entries[1].start;
        }
        result
    }
    fn root_anchor(&mut self, node: &crate::org::structure::Raw, prefix: &mut bool) {
        if !*prefix || node.kind == StructureKind::Section {
            return;
        }
        if node.kind == StructureKind::Comment {
            self.entries[0].anchor = node.source.end;
        } else {
            *prefix = false;
        }
    }
    fn metadata(&mut self, node: &crate::org::structure::Raw, text: &Rope, owner: usize) {
        if node.source.start != self.entries[owner].anchor {
            return;
        }
        let malformed = node.kind == StructureKind::Malformed;
        let ordinary_properties = node.kind == StructureKind::Drawer
            && node.name.as_ref().is_some_and(|r| {
                text.byte_slice(r.clone())
                    .to_string()
                    .eq_ignore_ascii_case("PROPERTIES")
            });
        self.entries[owner].invalid |= malformed || ordinary_properties;
    }
    fn collect(&mut self, node: &crate::org::structure::Raw, text: &Rope, owner: usize) -> usize {
        match node.kind {
            StructureKind::Heading { .. } => {
                let entry = Entry {
                    start: node.source.start,
                    end: node.source.end,
                    anchor: node.opening.as_ref().expect("heading opening").end,
                    parent: Some(owner),
                    invalid: false,
                    drawer: None,
                    records: Vec::new(),
                };
                self.entries.push(entry);
                return self.entries.len() - 1;
            }
            StructureKind::Planning => {
                self.entries[owner].anchor = node.source.end;
                self.contexts.push(node.source.clone());
            }
            StructureKind::PropertyDrawer => self.drawer(node, text, owner),
            StructureKind::Property { additive } => self.property(node, text, owner, additive),
            StructureKind::Keyword => self.directive(node, text),
            _ => {}
        }
        owner
    }
    fn drawer(&mut self, node: &crate::org::structure::Raw, text: &Rope, owner: usize) {
        let open = node.opening.as_ref().expect("drawer opening");
        let close = node.closing.as_ref().expect("property drawer closing");
        let opening = text.byte_slice(open.clone()).to_string();
        let indent: String = opening
            .chars()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .collect();
        let source = open.start..close.end;
        self.contexts.push(source.clone());
        self.entries[owner].drawer = Some(Drawer {
            source,
            closing: close.start,
            indent,
        });
    }
    fn property(
        &mut self,
        node: &crate::org::structure::Raw,
        text: &Rope,
        owner: usize,
        additive: bool,
    ) {
        let Some(name) = &node.name else { return };
        let raw = text.byte_slice(name.clone()).to_string();
        let Ok(key) = property_name(&raw) else {
            self.complete = false;
            return;
        };
        self.push(Record {
            key: key.trim_end_matches('+').into(),
            additive,
            source: node.source.clone(),
            value: node.content.clone(),
            owner,
            directive: false,
        });
    }
    fn directive(&mut self, node: &crate::org::structure::Raw, text: &Rope) {
        let Some(name) = &node.name else { return };
        if !text
            .byte_slice(name.clone())
            .to_string()
            .eq_ignore_ascii_case("PROPERTY")
        {
            return;
        }
        let value = text.byte_slice(node.content.clone()).to_string();
        let end = value.find(char::is_whitespace).unwrap_or(value.len());
        let Ok(key) = property_name(&value[..end]) else {
            return;
        };
        let tail = &value[end..];
        let start = value.len() - tail.trim_start_matches([' ', '\t']).len();
        self.contexts.push(node.source.clone());
        self.push(Record {
            key: key.trim_end_matches('+').into(),
            additive: key.ends_with('+'),
            source: node.source.clone(),
            value: node.content.start + start..node.content.end,
            owner: 0,
            directive: true,
        });
    }
    fn push(&mut self, record: Record) {
        self.entries[record.owner].records.push(self.records.len());
        self.records.push(record);
    }
}
