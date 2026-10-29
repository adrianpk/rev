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

pub(super) fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err("Org export preparation cancelled or expired".into());
    }
    Ok(())
}

// Native parent indices precede their children. Derive literal ownership once
// without repeated ancestor walks or treating drawer children as export actions.
pub(super) fn active<'a>(
    nodes: &'a [structure::Raw],
    cancel: &Cancellation,
) -> Result<impl Iterator<Item = &'a structure::Raw>, String> {
    let mut blocked: Vec<bool> = Vec::with_capacity(nodes.len());
    for node in nodes {
        check(cancel)?;
        let parent = node.parent.is_some_and(|p| blocked[p]);
        blocked.push(
            parent
                || matches!(
                    node.kind,
                    StructureKind::Drawer | StructureKind::PropertyDrawer
                ),
        );
    }
    Ok(nodes
        .iter()
        .enumerate()
        .filter_map(move |(i, n)| (!blocked[i]).then_some(n)))
}
