use super::*;
pub(super) fn nodes(text: &Rope, cancel: &Cancellation) -> Result<Vec<structure::Raw>, String> {
    let document = structure::observe(text.clone(), super::super::Limits::default(), cancel);
    if document.coverage != super::super::Coverage::Complete {
        return Err("Incomplete native Org export structure".into());
    }
    if document
        .nodes
        .iter()
        .any(|n| n.kind == StructureKind::Malformed)
    {
        return Err("Malformed native Org export structure".into());
    }
    Ok(document.nodes)
}
pub(super) fn keyword(text: &Rope, node: &structure::Raw) -> Option<(String, String)> {
    if node.kind != StructureKind::Keyword {
        return None;
    }
    Some((
        text.byte_slice(node.name.clone()?)
            .to_string()
            .to_ascii_uppercase(),
        text.byte_slice(node.content.clone()).to_string(),
    ))
}
