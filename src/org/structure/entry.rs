//! Shared bounded heading-level and newline normalization for native moves.
use crate::org::{Cancellation, Coverage, Limits, StructureKind, structure};
use ropey::Rope;
pub(in crate::org) fn relevel(
    draft: &Rope,
    level: usize,
    newline: &str,
    cancel: &Cancellation,
) -> Result<String, String> {
    let document = structure::observe(draft.clone(), Limits::default(), cancel);
    if document.coverage != Coverage::Complete
        || document
            .nodes
            .iter()
            .any(|n| n.kind == StructureKind::Malformed)
    {
        return Err("Org entry has incomplete or malformed Org structure".into());
    }
    let headings: Vec<_> = document
        .nodes
        .iter()
        .filter_map(|n| match n.kind {
            StructureKind::Heading { level } => Some((n.source.start, level)),
            _ => None,
        })
        .collect();
    let Some(&(0, first)) = headings.first() else {
        return Err("Org entry must start with a heading".into());
    };
    let mut text = String::new();
    let mut offset = 0;
    for (at, old) in headings {
        if cancel.is_cancelled() {
            return Err("Org entry normalization cancelled".into());
        }
        let relative = old
            .checked_sub(first)
            .ok_or("Org entry contains a heading above its root")?;
        let new = level
            .checked_add(relative)
            .filter(|n| *n <= 256)
            .ok_or("Org entry exceeds 256 heading levels")?;
        text.push_str(&draft.byte_slice(offset..at).to_string());
        text.push_str(&"*".repeat(new));
        offset = at + old;
    }
    text.push_str(&draft.byte_slice(offset..).to_string());
    Ok(text.replace("\r\n", "\n").replace('\n', newline))
}
