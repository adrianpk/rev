use ropey::RopeSlice;
use tree_sitter::Node;

use crate::completion::context::Context;
use crate::config::Grammar;

use super::Syntax;

impl Syntax {
    pub(crate) fn completion_context(&self, text: RopeSlice<'_>, byte: usize) -> Context {
        let Some(parsed) = &self.parsed else {
            return Context::Unsupported;
        };
        let Some(tree) = &parsed.tree else {
            return Context::Unsupported;
        };
        if matches!(parsed.grammar, Grammar::Json | Grammar::Toml) {
            return Context::Unsupported;
        }
        if !matches!(parsed.grammar, Grammar::Html | Grammar::Css) {
            return Context::Names;
        }
        let mut node = tree
            .root_node()
            .descendant_for_byte_range(byte.saturating_sub(1), byte);
        let mut error_start = None;
        for _ in 0..128 {
            let Some(current) = node else {
                break;
            };
            if current.is_error() {
                error_start = Some(current.start_byte());
            }
            let context = if parsed.grammar == Grammar::Html {
                html(current, byte)
            } else {
                css(current, text, byte)
            };
            if let Some(context) = context {
                return context;
            }
            node = current.parent();
        }
        html_prefix(parsed.grammar, text, byte, error_start)
    }
}

fn html(node: Node<'_>, byte: usize) -> Option<Context> {
    if [
        "attribute_value",
        "quoted_attribute_value",
        "comment",
        "raw_text",
    ]
    .contains(&node.kind())
    {
        return Some(Context::Unsupported);
    }
    match node.kind() {
        "tag_name" | "erroneous_end_tag_name" => Some(Context::Tag),
        "attribute_name" => Some(Context::Attribute),
        "start_tag" | "self_closing_tag" => Some(html_tag(node, byte)),
        "end_tag" | "erroneous_end_tag" => Some(if closed_tag(node, byte) {
            Context::Unsupported
        } else {
            Context::Tag
        }),
        _ => None,
    }
}

fn html_tag(node: Node<'_>, byte: usize) -> Context {
    if closed_tag(node, byte) {
        return Context::Unsupported;
    }
    let mut cursor = node.walk();
    let tag = node
        .named_children(&mut cursor)
        .find(|child| child.kind() == "tag_name");
    if tag.is_some_and(|tag| byte > tag.end_byte()) {
        Context::Attribute
    } else {
        Context::Tag
    }
}

fn html_prefix(
    grammar: Grammar,
    text: RopeSlice<'_>,
    byte: usize,
    error_start: Option<usize>,
) -> Context {
    if grammar != Grammar::Html {
        return Context::Unsupported;
    }
    // Recovery is limited to the current identifier and its literal delimiter
    // within this error node. Declaration names still come only from queries.
    let end = text.byte_to_char(byte);
    let mut start = end;
    let lower = end.saturating_sub(crate::completion::MAX_LABEL);
    while start > lower && html_name(text.char(start - 1)) {
        start -= 1;
    }
    if start > 0 && text.char(start - 1) == '/' {
        start -= 1;
    }
    if start > 0
        && text.char(start - 1) == '<'
        && error_start.is_none_or(|error| error == text.char_to_byte(start - 1))
    {
        Context::Tag
    } else {
        Context::Unsupported
    }
}

fn html_name(ch: char) -> bool {
    ch.is_alphanumeric() || ['_', '-', ':'].contains(&ch)
}

fn css(node: Node<'_>, text: RopeSlice<'_>, byte: usize) -> Option<Context> {
    let kind = node.kind();
    if ["comment", "string_value"].contains(&kind) {
        return Some(Context::Unsupported);
    }
    if [
        "class_name",
        "id_name",
        "tag_name",
        "selectors",
        "stylesheet",
    ]
    .contains(&kind)
    {
        return Some(Context::Selector);
    }
    match kind {
        "property_name" | "block" => Some(Context::Property),
        "declaration" => Some(css_declaration(node, byte)),
        "plain_value" | "call_expression" => Some(css_value(node, text)),
        _ => None,
    }
}

fn css_declaration(node: Node<'_>, byte: usize) -> Context {
    let mut cursor = node.walk();
    let name = node
        .named_children(&mut cursor)
        .find(|child| child.kind() == "property_name");
    if name.is_some_and(|name| byte <= name.end_byte()) {
        Context::Property
    } else {
        Context::Unsupported
    }
}

fn css_value(mut node: Node<'_>, text: RopeSlice<'_>) -> Context {
    for _ in 0..128 {
        if node.kind() == "call_expression" {
            let mut cursor = node.walk();
            let function = node
                .named_children(&mut cursor)
                .find(|child| child.kind() == "function_name");
            if function.is_some_and(|name| text.byte_slice(name.byte_range()) == "var") {
                return Context::CustomProperty;
            }
        }
        let Some(parent) = node.parent() else {
            break;
        };
        node = parent;
    }
    Context::Unsupported
}

#[cfg(test)]
mod tests {
    use super::*;

    // Closing-tag error recovery still identifies a structural tag-name target.
    #[test]
    fn recovered_end_tag() {
        let mut buffers = crate::buffer::Buffers::new().unwrap();
        let document = "<custom-card></custom-card>\n</cus>";
        let id = buffers.create(document).unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut syntax = Syntax::bundled(Grammar::Html);
        let text = buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap();
        syntax.update(text, buffer.stamp());
        let tree = syntax.parsed.as_ref().unwrap().tree.as_ref().unwrap();
        assert_eq!(
            syntax.completion_context(text, document.len() - 1),
            Context::Tag,
            "{}",
            tree.root_node().to_sexp()
        );
    }
}

fn closed_tag(node: Node<'_>, byte: usize) -> bool {
    node.end_byte() == byte
        && node
            .child(node.child_count().saturating_sub(1))
            .is_some_and(|last| !last.is_missing() && [">", "/>"].contains(&last.kind()))
}
