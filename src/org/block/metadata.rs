use super::*;
pub(super) struct Metadata {
    pub language: String,
    pub switches: String,
    pub name: Option<String>,
    pub headers: Headers,
}
pub(super) fn observe(
    text: &Rope,
    document: &structure::Document,
    node: &structure::Raw,
    cancel: &Cancellation,
) -> Result<Metadata, String> {
    let opening = node.opening.clone().ok_or("Source block has no opening")?;
    let (language, switches, local) = headers::opening(&text.byte_slice(opening).to_string())?;
    let mut headers = inherited(text, document, node, &language, cancel)?;
    let name = headers::affiliated(text, node, &mut headers, cancel)?;
    unique(text, document, name.as_deref(), cancel)?;
    headers.apply(&local, cancel)?;
    Ok(Metadata {
        language,
        switches,
        name,
        headers,
    })
}
fn inherited(
    text: &Rope,
    document: &structure::Document,
    node: &structure::Raw,
    language: &str,
    cancel: &Cancellation,
) -> Result<Headers, String> {
    let layers = super::super::property::Properties::observe_nodes(
        document.nodes.iter(),
        true,
        text,
        cancel,
    )
    .header_layers(text, node.source.start, language)?;
    let mut headers = Headers::defaults();
    for layer in layers {
        headers.apply(&layer, cancel)?;
    }
    Ok(headers)
}
fn unique(
    text: &Rope,
    document: &structure::Document,
    name: Option<&str>,
    cancel: &Cancellation,
) -> Result<(), String> {
    let Some(name) = name else {
        return Ok(());
    };
    let mut count = 0;
    for node in &document.nodes {
        headers::check(cancel)?;
        if node.kind == StructureKind::Block(BlockKind::Source) {
            count += name_count(text, node, name)?;
        }
        if count > 1 {
            return Err("Source block name is ambiguous in this document".into());
        }
    }
    Ok(())
}
fn name_count(text: &Rope, node: &structure::Raw, name: &str) -> Result<usize, String> {
    let Some(range) = &node.affiliated else {
        return Ok(0);
    };
    if range.len() > 65536 {
        return Err("Source affiliations exceed 64 KiB".into());
    }
    Ok(text
        .byte_slice(range.clone())
        .lines()
        .filter(|line| {
            let line = line.to_string();
            line.trim()
                .strip_prefix("#+")
                .and_then(|s| s.split_once(':'))
                .is_some_and(|(key, value)| {
                    key.eq_ignore_ascii_case("NAME") && value.trim() == name
                })
        })
        .count())
}
