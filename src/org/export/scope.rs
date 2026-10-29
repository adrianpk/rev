use super::*;
use crate::buffer::Buffer;
pub(super) fn capture(buffer: &Buffer, scope: &Scope) -> Result<Range<usize>, String> {
    match scope {
        Scope::Region(region) => {
            Source::new(buffer)
                .text(region)
                .map_err(|e| e.to_string())?;
            let c = crate::coordinates::Coordinates::new(buffer).map_err(|e| e.to_string())?;
            let r = &region.span().range;
            Ok(c.to_byte(crate::coordinates::Scalar(r.start))
                .map_err(|e| e.to_string())?
                .0
                ..c.to_byte(crate::coordinates::Scalar(r.end))
                    .map_err(|e| e.to_string())?
                    .0)
        }
        Scope::Subtree(byte) => {
            buffer
                .byte_span(byte.0..byte.0)
                .map_err(|e| e.to_string())?;
            Ok(byte.0..byte.0)
        }
        Scope::Buffer => Ok(0..buffer.len_bytes()),
    }
}
pub(super) fn select(
    nodes: &[structure::Raw],
    range: Range<usize>,
    scope: &Scope,
) -> Result<Range<usize>, String> {
    if !matches!(scope, Scope::Subtree(_)) {
        return Ok(range);
    }
    nodes
        .iter()
        .filter(|n| matches!(n.kind, StructureKind::Heading { .. }))
        .rfind(|n| n.source.start <= range.start && range.start < n.source.end)
        .map(|n| n.source.clone())
        .ok_or_else(|| "No native Org subtree at export point".into())
}

pub(super) fn region(nodes: &[structure::Raw], range: &Range<usize>) -> Result<(), String> {
    for n in nodes {
        let literal = matches!(
            n.kind,
            StructureKind::Block(_)
                | StructureKind::Drawer
                | StructureKind::PropertyDrawer
                | StructureKind::FixedWidth
                | StructureKind::Comment
        );
        if literal
            && n.source.start < range.end
            && range.start < n.source.end
            && (range.start > n.source.start || range.end < n.source.end)
        {
            return Err("Export selection splits a literal native Org form".into());
        }
    }
    Ok(())
}
