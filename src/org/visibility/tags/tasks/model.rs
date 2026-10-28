use super::*;
use crate::org::{StructureKind, structure::Raw};
use std::collections::BTreeMap;

#[derive(Debug, Default, PartialEq, Eq)]
pub(in super::super) struct Metadata {
    pub(in super::super) entries: Vec<Entry>,
}
#[derive(Debug, PartialEq, Eq)]
pub(in super::super) struct Entry {
    pub planning: Option<Range<usize>>,
    pub anchor: usize,
    pub drawers: Vec<Drawer>,
}
#[derive(Debug, PartialEq, Eq)]
pub(in super::super) struct Drawer {
    pub name: Range<usize>,
    pub body: Range<usize>,
}
impl Metadata {
    pub fn observe(structure: &Structure, cancel: &Cancellation) -> Self {
        let mut result = Self::default();
        let mut owners = BTreeMap::new();
        for (id, node) in structure.raw_nodes().enumerate() {
            if cancel.is_cancelled()
                || matches!(node.kind, StructureKind::Heading { .. })
                    && result.entries.len() == 4096
            {
                break;
            }
            match node.kind {
                StructureKind::Heading { .. } => {
                    let index = result.entries.len();
                    result.entries.push(Entry {
                        planning: None,
                        anchor: node.opening.as_ref().expect("heading").end,
                        drawers: Vec::new(),
                    });
                    owners.insert(id, index);
                }
                StructureKind::Section => {
                    if let Some(owner) = node.parent.and_then(|p| owners.get(&p).copied()) {
                        owners.insert(id, owner);
                    }
                }
                _ => {
                    if let Some(owner) = node.parent.and_then(|p| owners.get(&p).copied()) {
                        result.entries[owner].collect(node);
                    }
                }
            }
        }
        result
    }
}
impl Entry {
    fn collect(&mut self, node: &Raw) {
        match node.kind {
            StructureKind::Planning => {
                self.planning = Some(node.content.clone());
                self.anchor = node.source.end;
            }
            StructureKind::PropertyDrawer => {
                self.anchor = node.closing.as_ref().expect("closed drawer").end;
            }
            StructureKind::Drawer => {
                if let Some(name) = &node.name {
                    self.drawers.push(Drawer {
                        name: name.clone(),
                        body: node.content.clone(),
                    });
                }
            }
            _ => (),
        }
    }
}
