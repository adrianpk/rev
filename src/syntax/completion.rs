use std::ops::ControlFlow;
use std::time::Instant;

use ropey::RopeSlice;
use tree_sitter::{Query, QueryCursor, QueryCursorOptions, StreamingIterator};

use crate::buffer::Stamp;
use crate::completion::{MAX_LABEL, MAX_SYMBOLS, Symbol};

use super::{Source, Syntax};

const QUERY: &str = r#"
(function_item name: (identifier) @function)
(struct_item name: (type_identifier) @type)
(enum_item name: (type_identifier) @type)
(trait_item name: (type_identifier) @type)
(type_item name: (type_identifier) @type)
(mod_item name: (identifier) @module)
(const_item name: (identifier) @constant)
(static_item name: (identifier) @constant)
(let_declaration pattern: (identifier) @binding)
(parameter pattern: (identifier) @parameter)
(field_declaration name: (field_identifier) @field)
"#;

pub(super) fn query(language: &tree_sitter::Language) -> Option<Query> {
    Query::new(language, QUERY).ok()
}

impl Syntax {
    pub(crate) fn declarations(
        &self,
        text: RopeSlice<'_>,
        stamp: Stamp,
        deadline: Instant,
    ) -> Result<Vec<Symbol>, String> {
        if self.mode(stamp) != "Rust" {
            return Err("Native Rust completion is unavailable for this buffer".into());
        }
        let parsed = self.parsed.as_ref().expect("current Rust parser");
        let tree = parsed.tree.as_ref().expect("current Rust tree");
        let query = parsed
            .completion
            .as_ref()
            .ok_or("Rust completion query is unavailable")?;
        let mut cursor = QueryCursor::new();
        cursor.set_match_limit(4096);
        let mut progress = |_: &tree_sitter::QueryCursorState| expired(deadline);
        let options = QueryCursorOptions::new().progress_callback(&mut progress);
        let mut captures =
            cursor.captures_with_options(query, tree.root_node(), Source(text), options);
        let mut symbols = Vec::new();
        while let Some((matched, index)) = captures.next() {
            if Instant::now() >= deadline || symbols.len() >= MAX_SYMBOLS {
                return Err("Native completion work or symbol limit reached".into());
            }
            let capture = matched.captures()[*index];
            if let Some(symbol) = symbol(
                capture.node,
                query.capture_names()[capture.index as usize],
                text,
            ) {
                symbols.push(symbol);
            }
        }
        if cursor.did_exceed_match_limit() || Instant::now() >= deadline {
            return Err("Native completion work limit reached".into());
        }
        Ok(symbols)
    }
}

fn expired(deadline: Instant) -> ControlFlow<()> {
    if Instant::now() >= deadline {
        ControlFlow::Break(())
    } else {
        ControlFlow::Continue(())
    }
}

fn symbol(node: tree_sitter::Node<'_>, kind: &str, text: RopeSlice<'_>) -> Option<Symbol> {
    if node.has_error()
        || node.is_missing()
        || node.parent().is_some_and(|parent| parent.has_error())
    {
        return None;
    }
    let bytes = node.byte_range();
    if bytes.len() > MAX_LABEL {
        return None;
    }
    let kind = match kind {
        "function" => "function",
        "type" => "type",
        "module" => "module",
        "constant" => "constant",
        "binding" => "binding",
        "parameter" => "parameter",
        "field" => "field",
        _ => return None,
    };
    Some(Symbol {
        name: text.byte_slice(bytes.clone()).to_string(),
        kind,
        byte: bytes.start,
    })
}
