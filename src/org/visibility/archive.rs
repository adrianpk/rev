use super::*;
pub(super) fn declaration(
    structure: &Structure,
    text: &ropey::Rope,
    cancel: &Cancellation,
) -> Result<Option<String>, String> {
    let mut value = None;
    for node in structure
        .raw_nodes()
        .filter(|n| n.kind == crate::org::StructureKind::Keyword)
    {
        if cancel.is_cancelled() {
            return Err("Archive declaration observation cancelled".into());
        }
        let Some(name) = &node.name else {
            continue;
        };
        if text
            .byte_slice(name.clone())
            .to_string()
            .eq_ignore_ascii_case("ARCHIVE")
        {
            if node.content.len() > 4096 {
                return Err("Archive location exceeds 4096 bytes".into());
            }
            value = Some(
                text.byte_slice(node.content.clone())
                    .to_string()
                    .trim()
                    .into(),
            );
        }
    }
    Ok(value)
}
