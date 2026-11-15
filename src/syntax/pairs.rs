use crate::buffer::Stamp;
use crate::pairs::{Context, Hint, Kind, MAX_DEPTH, Profile};

impl super::Syntax {
    /// Observe one current path with finite descent. Neither a node nor a cursor
    /// escapes this call, and external highlight labels never define quote rules.
    pub(crate) fn pair_hint(&self, stamp: Stamp, byte: usize, profile: Profile) -> Option<Hint> {
        if self.stamp != Some(stamp) {
            return None;
        }
        let tree = self.parsed.as_ref()?.tree.as_ref()?;
        let mut cursor = tree.walk();
        let mut context = Context::Code;
        for nodes in 1..=MAX_DEPTH {
            let node = cursor.node();
            if !node.byte_range().contains(&byte) {
                return Some(Hint {
                    context,
                    nodes,
                    limited: false,
                });
            }
            if node.is_error() || node.is_missing() {
                return None;
            }
            if let Some(observed) = literal(node.kind(), profile) {
                context = observed;
            }
            if cursor.goto_first_child_for_byte(byte).is_none() {
                return Some(Hint {
                    context,
                    nodes,
                    limited: false,
                });
            }
        }
        Some(Hint {
            context,
            nodes: MAX_DEPTH,
            limited: true,
        })
    }
}

fn literal(kind: &str, profile: Profile) -> Option<Context> {
    if kind == "char_literal" {
        return Some(Context::String(Kind::SingleQuote));
    }
    if kind == "raw_string_literal" && profile == Profile::Go {
        return Some(Context::String(Kind::Backtick));
    }
    if kind.contains("comment") {
        return Some(Context::Comment);
    }
    if [
        "raw_string_literal",
        "template_string",
        "regex",
        "raw_text",
        "script_element",
        "style_element",
        "lifetime",
        "jsx_text",
        "jsx_element",
        "jsx_self_closing_element",
    ]
    .contains(&kind)
    {
        return Some(Context::Literal);
    }
    if kind.contains("string") || kind == "quoted_attribute_value" {
        return Some(Context::String(Kind::DoubleQuote));
    }
    None
}
