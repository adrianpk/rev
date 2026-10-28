use super::{Drawer, Heading, Outline};
use crate::org::{Cancellation, Coverage, Limit, Structure, StructureKind};

pub(super) fn observe(structure: &Structure, cancel: &Cancellation) -> Outline {
    let mut result = Outline {
        stamp: structure.stamp(),
        coverage: structure.coverage(),
        headings: Vec::new(),
        drawers: Vec::new(),
        blocked: Vec::new(),
        blocked_eof: false,
        malformed: Vec::new(),
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
        block(&mut result, node, structure.covered_bytes().0);
        if node.kind == StructureKind::Malformed {
            result.malformed.push(node.source.clone());
        }
    }
    result.headings.sort_by_key(|h| h.start);
    result.drawers.sort_by_key(|d| d.start);
    parents(&mut result.headings);
    merge(&mut result.blocked);
    merge(&mut result.malformed);
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
            title: node.content.clone(),
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

// Deeper source forms cannot fall through to a destructive ancestor insertion.
// These offsets share the existing source-node bound and live only in the worker result.
fn block(result: &mut Outline, node: &crate::org::structure::Raw, eof: usize) {
    use StructureKind::*;
    if matches!(node.kind, Heading { .. } | Section | Paragraph) {
        return;
    }
    let end = node
        .closing
        .as_ref()
        .map_or(node.source.end, |close| close.end);
    result.blocked_eof |= node.source.end == eof && node.closing.is_none();
    result.blocked.push(node.source.start..end);
}

fn merge(ranges: &mut Vec<std::ops::Range<usize>>) {
    ranges.sort_by_key(|range| range.start);
    let mut merged: Vec<std::ops::Range<usize>> = Vec::new();
    for range in ranges.drain(..) {
        if let Some(last) = merged.last_mut().filter(|last| range.start <= last.end) {
            last.end = last.end.max(range.end);
        } else {
            merged.push(range);
        }
    }
    *ranges = merged;
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
