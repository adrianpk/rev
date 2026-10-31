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
    pub(crate) fn completion_allowed(
        &self,
        byte: usize,
        policy: &crate::config::Completion,
    ) -> bool {
        let Some(tree) = self.parsed.as_ref().and_then(|p| p.tree.as_ref()) else {
            return false;
        };
        let mut node = tree
            .root_node()
            .descendant_for_byte_range(byte.saturating_sub(1), byte);
        while let Some(current) = node {
            match current.kind() {
                "line_comment" | "block_comment" => return policy.in_comments,
                "string_literal" | "raw_string_literal" | "char_literal" => {
                    return policy.in_strings;
                }
                _ => node = current.parent(),
            }
        }
        true
    }
    #[cfg(test)]
    pub(crate) fn declarations(
        &self,
        text: RopeSlice<'_>,
        stamp: Stamp,
        deadline: Instant,
    ) -> Result<Vec<Symbol>, String> {
        self.declarations_with(
            text,
            stamp,
            deadline,
            &std::sync::atomic::AtomicBool::new(false),
        )
    }

    pub(crate) fn declarations_with(
        &self,
        text: RopeSlice<'_>,
        stamp: Stamp,
        deadline: Instant,
        cancel: &std::sync::atomic::AtomicBool,
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
        let mut progress = |_: &tree_sitter::QueryCursorState| expired(deadline, cancel);
        let options = QueryCursorOptions::new().progress_callback(&mut progress);
        let mut captures =
            cursor.captures_with_options(query, tree.root_node(), Source(text), options);
        let mut symbols = Vec::new();
        while let Some((matched, index)) = captures.next() {
            if stopped(deadline, cancel) || symbols.len() >= MAX_SYMBOLS {
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
        if cursor.did_exceed_match_limit() || stopped(deadline, cancel) {
            return Err("Native completion work limit reached".into());
        }
        Ok(symbols)
    }
}

fn stopped(deadline: Instant, cancel: &std::sync::atomic::AtomicBool) -> bool {
    Instant::now() >= deadline || cancel.load(std::sync::atomic::Ordering::Acquire)
}

fn expired(deadline: Instant, cancel: &std::sync::atomic::AtomicBool) -> ControlFlow<()> {
    if stopped(deadline, cancel) {
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
    Some(Symbol {
        name: text.byte_slice(bytes.clone()).to_string(),
        kind: category(kind)?,
        byte: bytes.start,
    })
}

fn category(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "function" => "function",
        "type" => "type",
        "module" => "module",
        "constant" => "constant",
        "binding" => "binding",
        "parameter" => "parameter",
        "field" => "field",
        _ => return None,
    })
}
