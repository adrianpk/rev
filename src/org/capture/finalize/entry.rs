use super::*;
pub(super) use crate::org::structure::entry::relevel as text;
use crate::org::{Coverage, Limits, StructureKind, structure};

pub(super) fn first(source: &Rope, cancel: &Cancellation) -> Result<usize, String> {
    let document = structure::observe(source.clone(), Limits::default(), cancel);
    if document.coverage != Coverage::Complete {
        return Err("Capture source observation is incomplete".into());
    }
    Ok(document
        .nodes
        .iter()
        .find(|n| matches!(n.kind, StructureKind::Heading { .. }))
        .map_or(source.len_bytes(), |n| n.source.start))
}

/// Prepending body text must not detach the heading's planning or drawers.
pub(super) fn body_start(resolved: &Resolved, cancel: &Cancellation) -> Result<usize, String> {
    let source = resolved.source.snapshot();
    let document = structure::observe(source.clone(), Limits::default(), cancel);
    if document.coverage != Coverage::Complete {
        return Err("Capture body observation is incomplete".into());
    }
    let body = &resolved.target.body;
    let mut at = body.start.0;
    for node in document.nodes.iter().filter(|n| {
        n.source.start >= body.start.0 && n.source.end <= body.end.0 && metadata(n.kind)
    }) {
        if node.source.start < at {
            continue;
        }
        if source
            .byte_slice(at..node.source.start)
            .chars()
            .any(|c| !c.is_whitespace())
        {
            break;
        }
        at = node.source.end;
    }
    Ok(at)
}
fn metadata(kind: StructureKind) -> bool {
    matches!(
        kind,
        StructureKind::Planning | StructureKind::PropertyDrawer | StructureKind::Drawer
    )
}
