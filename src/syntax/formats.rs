//! Structural key targets from the retained tree, with bounded incomplete-token recovery.
use super::Syntax;
use crate::completion::{MAX_LABEL, context::Context, literal};
use crate::config::Grammar;
use ropey::RopeSlice;
use std::ops::Range;
use tree_sitter::Node;

pub(crate) struct Target {
    pub range: Range<usize>,
    pub prefix: String,
    pub original: String,
    pub context: Context,
}

impl Syntax {
    pub(crate) fn format_target(&self, text: RopeSlice<'_>, byte: usize) -> Option<Target> {
        let parsed = self.parsed.as_ref()?;
        if !matches!(parsed.grammar, Grammar::Json | Grammar::Toml) {
            return None;
        }
        let tree = parsed.tree.as_ref()?;
        if parsed.grammar == Grammar::Toml && broken_value(tree.root_node(), byte) {
            return None;
        }
        let node = tree
            .root_node()
            .descendant_for_byte_range(byte.saturating_sub(1), byte)?;
        native(node, text, byte, parsed.grammar)
            .unwrap_or_else(|| recovery(node, text, byte, parsed.grammar))
    }
}

fn native(
    mut node: Node<'_>,
    text: RopeSlice<'_>,
    byte: usize,
    grammar: Grammar,
) -> Option<Option<Target>> {
    for _ in 0..128 {
        if node.kind() == "comment" {
            return Some(None);
        }
        if node.kind() == "string"
            && let Some(target) = string_node(node, text, byte, grammar)
        {
            return Some(target);
        }
        if grammar == Grammar::Toml
            && let Some(target) = toml_node(node, text, byte)
        {
            return Some(target);
        }
        node = node.parent()?;
    }
    Some(None)
}

fn string_node(
    node: Node<'_>,
    text: RopeSlice<'_>,
    byte: usize,
    grammar: Grammar,
) -> Option<Option<Target>> {
    let parent = node.parent()?;
    if grammar == Grammar::Json && parent.child_by_field_name("key") == Some(node) {
        return Some(key(node.byte_range(), text, byte, grammar));
    }
    (!parent.is_error()).then_some(None)
}

fn toml_node(node: Node<'_>, text: RopeSlice<'_>, byte: usize) -> Option<Option<Target>> {
    if node.is_error() && error_value(node, byte) {
        return Some(None);
    }
    if ["table", "table_array_element"].contains(&node.kind()) {
        return header(node, text, byte).map(Some);
    }
    if ["bare_key", "quoted_key"].contains(&node.kind()) && pair_key(node) {
        return Some(key(node.byte_range(), text, byte, Grammar::Toml));
    }
    (node.kind() == "array").then_some(None)
}

fn broken_value(mut node: Node<'_>, byte: usize) -> bool {
    // The scanner can consume an unfinished multiline body without returning
    // child nodes for that body. Its final ERROR retains the value opener.
    for _ in 0..128 {
        if node.start_byte() > byte {
            return false;
        }
        if node.is_error() {
            return error_value(node, byte);
        }
        let Some(last) = node
            .child_count()
            .checked_sub(1)
            .and_then(|last| node.child(last))
        else {
            return false;
        };
        node = last;
    }
    true
}

fn error_value(node: Node<'_>, byte: usize) -> bool {
    let mut value = false;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).take(128) {
        if child.start_byte() > byte {
            break;
        }
        if child.kind() == "=" {
            value = true;
            continue;
        }
        if value && ["\"\"\"", "'''", "\"", "'", "["].contains(&child.kind()) {
            return true;
        }
        if child.is_named() || ["{", ","].contains(&child.kind()) {
            value = false;
        }
    }
    node.child_count() > 128
}

fn pair_key(mut node: Node<'_>) -> bool {
    for _ in 0..128 {
        let Some(parent) = node.parent() else {
            return false;
        };
        if parent.kind() == "pair" {
            return parent.named_child(0) == Some(node);
        }
        if parent.kind() != "dotted_key" {
            return false;
        }
        node = parent;
    }
    false
}

fn header(node: Node<'_>, text: RopeSlice<'_>, byte: usize) -> Option<Target> {
    let name = node.named_child(0)?;
    if byte < name.start_byte() || byte > name.end_byte() {
        return None;
    }
    table(name.byte_range(), text, byte)
}

fn key(range: Range<usize>, text: RopeSlice<'_>, byte: usize, grammar: Grammar) -> Option<Target> {
    if range.len() > MAX_LABEL || !range.contains(&byte.saturating_sub(1)) {
        return None;
    }
    let raw = text.byte_slice(range.clone()).to_string();
    let prefix = text.byte_slice(range.start..byte).to_string();
    let prefix = prefix
        .strip_prefix(
            raw.chars()
                .next()
                .filter(|ch| ['"', '\''].contains(ch))
                .unwrap_or('\0'),
        )
        .unwrap_or(&prefix);
    let prefix = if raw.starts_with('\'') {
        prefix.to_owned()
    } else {
        literal::content(prefix, grammar)?
    };
    Some(Target {
        range,
        prefix,
        original: literal::decode(&raw, grammar).unwrap_or(raw),
        context: if grammar == Grammar::Json {
            Context::JsonKey
        } else {
            Context::TomlKey
        },
    })
}

fn table(range: Range<usize>, text: RopeSlice<'_>, byte: usize) -> Option<Target> {
    if range.len() > MAX_LABEL {
        return None;
    }
    let raw = text.byte_slice(range.clone()).to_string();
    let prefix = text.byte_slice(range.start..byte).to_string();
    Some(Target {
        range,
        prefix: literal::path(&prefix, true)?,
        original: literal::path(&raw, false).unwrap_or(raw),
        context: Context::TomlTable,
    })
}

fn recovery(node: Node<'_>, text: RopeSlice<'_>, byte: usize, grammar: Grammar) -> Option<Target> {
    let point = text.byte_to_char(byte);
    let line = text.line_to_char(text.char_to_line(point));
    if point - line > MAX_LABEL {
        return None;
    }
    let prefix = text.slice(line..point).to_string();
    if grammar == Grammar::Toml {
        return toml_recovery(text, byte, &prefix);
    }
    json_recovery(node, text, byte, &prefix)
}

fn json_recovery(node: Node<'_>, text: RopeSlice<'_>, byte: usize, prefix: &str) -> Option<Target> {
    let start = quote_start(prefix).unwrap_or(prefix.len());
    let from = byte - (prefix.len() - start);
    let delimiter = previous(text, from)?;
    if !['{', ','].contains(&delimiter) {
        return None;
    }
    if delimiter == ',' && !object(node) {
        return None;
    }
    let range = byte - (prefix.len() - start)..byte;
    if start == prefix.len() {
        return Some(Target {
            range,
            prefix: String::new(),
            original: String::new(),
            context: Context::JsonKey,
        });
    }
    key(range, text, byte, Grammar::Json)
}

fn previous(text: RopeSlice<'_>, byte: usize) -> Option<char> {
    let end = text.byte_to_char(byte);
    let start = end.saturating_sub(MAX_LABEL);
    text.slice(start..end)
        .chars_at(end - start)
        .reversed()
        .find(|ch| !ch.is_whitespace())
}

fn quote_start(prefix: &str) -> Option<usize> {
    let mut start = None;
    let mut escaped = false;
    for (byte, ch) in prefix.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if start.is_some() && ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            start = if start.is_some() { None } else { Some(byte) };
        }
    }
    start
}

fn object(mut node: Node<'_>) -> bool {
    let origin = node;
    for _ in 0..128 {
        match node.kind() {
            "object" => return true,
            "array" => return false,
            _ => {}
        }
        let Some(parent) = node.parent() else {
            return recovered_object(origin);
        };
        node = parent;
    }
    false
}

fn recovered_object(mut node: Node<'_>) -> bool {
    // An unfinished quoted key can turn the entire object into ERROR. Walk
    // bounded retained-tree siblings to its nearest explicit container opener;
    // complete nested values are single nodes and do not change this context.
    for _ in 0..128 {
        match node.kind() {
            "{" => return true,
            "[" => return false,
            _ => {}
        }
        let Some(previous) = node.prev_sibling().or_else(|| node.parent()) else {
            return false;
        };
        node = previous;
    }
    false
}

fn toml_recovery(text: RopeSlice<'_>, byte: usize, prefix: &str) -> Option<Target> {
    let trimmed = prefix.trim_start();
    let start = byte - trimmed.len();
    if trimmed.starts_with('[') {
        let brackets = if trimmed.starts_with("[[") { 2 } else { 1 };
        if trimmed.contains(']') {
            return None;
        }
        return table(start + brackets..byte, text, byte);
    }
    // Values/arrays/strings were rejected by native ancestry. A line-start key
    // (including an incomplete quoted key) must still precede its equals sign.
    let start = trimmed.rfind(['{', ',']).map_or(0, |offset| offset + 1);
    let raw = trimmed[start..].trim_start();
    if raw.contains('=') || raw.starts_with('#') {
        return None;
    }
    let raw = &raw[literal::tail(raw)..];
    let raw = raw.trim_start();
    let range = byte - raw.len()..byte;
    if raw.is_empty() {
        return Some(Target {
            range,
            prefix: String::new(),
            original: String::new(),
            context: Context::TomlKey,
        });
    }
    key(range, text, byte, Grammar::Toml)
}

pub(super) fn name(node: Node<'_>, kind: &str, text: RopeSlice<'_>) -> Option<String> {
    if kind == "toml-key" && !pair_key(node) {
        return None;
    }
    let raw = text.byte_slice(node.byte_range()).to_string();
    match kind {
        "json-key" => literal::decode(&raw, Grammar::Json),
        "toml-key" => literal::decode(&raw, Grammar::Toml),
        "table" => literal::path(&raw, false),
        _ => Some(raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The retained JSON error tree still admits a partial structural key and
    // preserves the independent complete pair before that editing position.
    #[test]
    fn incomplete_json() {
        for source in ["{\"sample\":1,\n \"sam|", "{\"sample\":1, \"|}"] {
            let byte = source.find('|').unwrap();
            let mut buffers = crate::buffer::Buffers::new().unwrap();
            let id = buffers.create(&source.replace('|', "")).unwrap();
            let buffer = buffers.get(id).unwrap();
            let text = buffer
                .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                .unwrap();
            let mut syntax = Syntax::bundled(Grammar::Json);
            syntax.update(text, buffer.stamp());
            let tree = syntax.parsed.as_ref().unwrap().tree.as_ref().unwrap();
            let target = syntax
                .format_target(text, byte)
                .map(|target| target.context);
            assert_eq!(
                target,
                Some(Context::JsonKey),
                "{}",
                tree.root_node().to_sexp()
            );
            let names = syntax
                .declarations(
                    text,
                    buffer.stamp(),
                    std::time::Instant::now() + crate::completion::WORK_TIME,
                )
                .unwrap();
            assert!(
                names.iter().any(|name| name.name == "sample"),
                "{}",
                tree.root_node().to_sexp()
            );
        }
    }
}

#[cfg(test)]
mod value_tests {
    use super::*;

    // Missing multiline/value-array terminators never reclassify their body as
    // a key position, including when the grammar recovers the body inside ERROR.
    #[test]
    fn unfinished_toml_values() {
        for source in ["sample=1\nvalue=\"\"\"\nsam", "sample=1\nvalue=[\n\"sam"] {
            let mut buffers = crate::buffer::Buffers::new().unwrap();
            let id = buffers.create(source).unwrap();
            let buffer = buffers.get(id).unwrap();
            let text = buffer
                .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                .unwrap();
            let mut syntax = Syntax::bundled(Grammar::Toml);
            syntax.update(text, buffer.stamp());
            let tree = syntax.parsed.as_ref().unwrap().tree.as_ref().unwrap();
            assert!(
                syntax.format_target(text, source.len()).is_none(),
                "{}",
                tree.root_node().to_sexp()
            );
        }
    }
}
