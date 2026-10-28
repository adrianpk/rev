use super::*;
use ropey::Rope;
pub(in crate::org) struct QueryHeader {
    pub title: String,
    pub word: String,
    pub tags: Vec<String>,
}
pub(in crate::org) fn query_header(
    node: &crate::org::structure::Raw,
    text: &Rope,
) -> Result<QueryHeader, String> {
    if node.content.len() > 4096 {
        return Err("Agenda heading exceeds 4096 bytes".into());
    }
    let mut tags = Tags::default();
    tags.header(node, text);
    let header = tags.headers.pop().expect("one raw heading");
    if !header.valid {
        return Err("Malformed or excessive Org heading tags".into());
    }
    Ok(QueryHeader {
        title: text.byte_slice(header.title).to_string(),
        word: header.word,
        tags: header.local.into_iter().map(|t| t.name).collect(),
    })
}
pub(in crate::org) fn query_file_tags(value: &str) -> Result<Vec<String>, String> {
    super::model::parse_suffix(value.trim(), 0)
        .map(|tags| tags.into_iter().map(|t| t.name).collect())
        .ok_or_else(|| "Malformed Org FILETAGS".into())
}
