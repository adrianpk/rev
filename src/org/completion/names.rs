use super::*;
use crate::org::{StructureKind, visibility::query_header};

pub(super) fn collect(
    structure: &Structure,
    text: &ropey::Rope,
    todo: &crate::config::Todo,
    kind: Kind,
    cancel: &Cancellation,
) -> Result<Vec<Symbol>, String> {
    let (workflow, tags) = crate::org::visibility::completion_names(structure, text, todo, cancel)?;
    let names = match kind {
        Kind::Todo => workflow
            .states
            .iter()
            .map(|state| state.name.clone())
            .collect(),
        Kind::Tag => tags,
        Kind::Property => crate::org::property::completion_names(structure, text, cancel)?,
        Kind::Directive => vocabulary::DIRECTIVES
            .iter()
            .map(|name| (*name).to_owned())
            .collect(),
        Kind::Link => vocabulary::LINKS
            .iter()
            .map(|name| (*name).to_owned())
            .collect(),
        Kind::Heading => return headings(structure, text, &workflow, cancel),
    };
    if names.len() > MAX_SYMBOLS {
        return Err("Native Org completion symbol limit reached".into());
    }
    names
        .into_iter()
        .map(|name| {
            check(cancel)?;
            symbol(name, kind, 0)
        })
        .collect()
}
fn headings(
    structure: &Structure,
    text: &ropey::Rope,
    workflow: &crate::org::task::Workflow,
    cancel: &Cancellation,
) -> Result<Vec<Symbol>, String> {
    let mut symbols = Vec::new();
    for node in structure
        .raw_nodes()
        .filter(|node| matches!(node.kind, StructureKind::Heading { .. }))
    {
        check(cancel)?;
        let header = query_header(node, text)?;
        let title = header.target(workflow)?;
        if title.is_empty() {
            continue;
        }
        if symbols.len() == MAX_SYMBOLS {
            return Err("Native Org completion symbol limit reached".into());
        }
        symbols.push(symbol(title.to_owned(), Kind::Heading, node.source.start)?);
    }
    Ok(symbols)
}
