use super::{Drawer, Heading, Outline};
use crate::org::{Cancellation, Coverage, Limit, Structure, StructureKind};

pub(super) fn observe(structure: &Structure, cancel: &Cancellation) -> Outline {
    let mut result = Outline {
        stamp: structure.stamp(),
        coverage: structure.coverage(),
        headings: Vec::new(),
        drawers: Vec::new(),
    };
    for node in structure.raw_nodes() {
        if cancel.is_cancelled() {
            result.coverage = Coverage::Cancelled;
            break;
        }
        if at_limit(&result, node) {
            result.coverage = Coverage::Incomplete(Limit::SourceNodes);
            break;
        }
        collect(&mut result, node);
    }
    result.headings.sort_by_key(|h| h.start);
    result.drawers.sort_by_key(|d| d.start);
    parents(&mut result.headings);
    result
}

fn at_limit(result: &Outline, node: &crate::org::structure::Raw) -> bool {
    matches!(
        node.kind,
        StructureKind::Heading { .. } | StructureKind::Drawer | StructureKind::PropertyDrawer
    ) && result.headings.len() + result.drawers.len() == 4096
}

fn collect(result: &mut Outline, node: &crate::org::structure::Raw) {
    let Some(open) = &node.opening else { return };
    match node.kind {
        StructureKind::Heading { level } => result.headings.push(Heading {
            start: open.start,
            body: open.end,
            end: node.source.end,
            section: node.source.end,
            parent: None,
            level,
        }),
        StructureKind::Drawer | StructureKind::PropertyDrawer => result.drawers.push(Drawer {
            start: open.start,
            body: node.content.clone(),
            end: node
                .closing
                .as_ref()
                .map_or(node.source.end, |close| close.end),
        }),
        _ => {}
    }
}
fn parents(headings: &mut [Heading]) {
    let mut stack: Vec<usize> = Vec::new();
    for index in 0..headings.len() {
        while stack
            .last()
            .is_some_and(|parent| headings[*parent].level >= headings[index].level)
        {
            stack.pop();
        }
        if let Some(parent) = stack.last().copied() {
            headings[index].parent = Some(parent);
            headings[parent].section = headings[parent].section.min(headings[index].start);
        }
        stack.push(index);
    }
}
