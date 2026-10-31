use std::collections::BTreeSet;
use std::ops::Range;

use tree_sitter::Node;

const BLOCKS: &[&str] = &[
    "block",
    "compound_statement",
    "for_expression",
    "for_statement",
    "if_expression",
    "if_statement",
    "while_expression",
    "while_statement",
    "match_arm",
    "expression_case",
    "type_case",
    "communication_case",
];
const FUNCTIONS: &[&str] = &[
    "function_item",
    "closure_expression",
    "function_definition",
    "function_declaration",
    "method_declaration",
    "func_literal",
];

/// Only explicit query scopes can admit local names; no token scan creates
/// lexical regions. Scope nodes are scratch data bounded independently of names.
#[derive(Default)]
pub(super) struct Scopes(BTreeSet<usize>);

impl Scopes {
    pub fn insert(&mut self, node: Node<'_>) -> Result<(), String> {
        if self.0.len() >= 8192 {
            return Err("Native completion scope limit reached".into());
        }
        self.0.insert(node.id());
        Ok(())
    }

    /// Fields remain observed member names without receiver type inference.
    pub fn declaration(&self, node: Node<'_>, kind: &str) -> Result<Option<Range<usize>>, String> {
        if kind == "field" {
            return Ok(None);
        }
        let mut parent = node.parent();
        let mut prototype = None;
        for _ in 0..128 {
            let Some(node) = parent else {
                return Ok(prototype);
            };
            if self.region(node, kind) {
                return Ok(Some(region_range(node, kind)));
            }
            if kind == "parameter"
                && node.kind() == "function_declarator"
                && self.0.contains(&node.id())
            {
                prototype = Some(node.byte_range());
            }
            parent = node.parent();
        }
        Err("Native completion scope nesting limit reached".into())
    }

    fn region(&self, node: Node<'_>, kind: &str) -> bool {
        self.0.contains(&node.id())
            && (BLOCKS.contains(&node.kind())
                || (kind != "function" && FUNCTIONS.contains(&node.kind())))
    }
}

fn region_range(node: Node<'_>, kind: &str) -> Range<usize> {
    if kind == "binding" {
        let field = match node.kind() {
            "if_expression" => Some("consequence"),
            "while_expression" | "for_expression" => Some("body"),
            _ => None,
        };
        if let Some(body) = field.and_then(|field| node.child_by_field_name(field)) {
            return insertion_range(body);
        }
    }
    insertion_range(node)
}

fn insertion_range(node: Node<'_>) -> Range<usize> {
    let mut range = node.byte_range();
    let body = node.child_by_field_name("body").unwrap_or(node);
    let last = body.child(body.child_count().saturating_sub(1) as u32);
    if last.is_some_and(|last| last.is_missing() && last.kind() == "}") {
        range.end = range.end.saturating_add(1);
    }
    range
}
