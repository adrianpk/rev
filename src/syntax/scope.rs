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
    "statement_block",
    "catch_clause",
    "for_in_statement",
];
const FUNCTIONS: &[&str] = &[
    "function_item",
    "closure_expression",
    "function_definition",
    "function_declaration",
    "method_declaration",
    "func_literal",
    "function_expression",
    "generator_function",
    "generator_function_declaration",
    "arrow_function",
    "method_definition",
    "method_signature",
];

/// Only explicit query scopes can admit local names; no token scan creates
/// lexical regions. Scope nodes are scratch data bounded independently of names.
#[derive(Default)]
pub(super) struct Scopes {
    nodes: BTreeSet<usize>,
    external: bool,
}

impl Scopes {
    pub fn new(external: bool) -> Self {
        Self {
            external,
            ..Self::default()
        }
    }
    pub fn insert(&mut self, node: Node<'_>) -> Result<(), String> {
        if self.nodes.len() >= 8192 {
            return Err("Native completion scope limit reached".into());
        }
        self.nodes.insert(node.id());
        Ok(())
    }

    /// Fields remain observed member names without receiver type inference.
    pub fn declaration(&self, node: Node<'_>, kind: &str) -> Result<Option<Range<usize>>, String> {
        if kind == "field" && !self.external {
            return Ok(None);
        }
        let mut parent = node.parent();
        let mut prototype = None;
        let mut function_scoped = false;
        for _ in 0..128 {
            let Some(node) = parent else {
                return Ok(prototype);
            };
            function_scoped |= kind == "binding" && node.kind() == "variable_declaration";
            if self.region(node, kind, function_scoped) {
                return Ok(Some(if self.external {
                    node.byte_range()
                } else {
                    region_range(node, kind)
                }));
            }
            if kind == "parameter"
                && node.kind() == "function_declarator"
                && self.nodes.contains(&node.id())
            {
                prototype = Some(node.byte_range());
            }
            parent = node.parent();
        }
        Err("Native completion scope nesting limit reached".into())
    }

    fn region(&self, node: Node<'_>, kind: &str, function_scoped: bool) -> bool {
        self.nodes.contains(&node.id())
            && (self.external
                || (!function_scoped && BLOCKS.contains(&node.kind()))
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
    let last = body.child(body.child_count().saturating_sub(1));
    if last.is_some_and(|last| last.is_missing() && last.kind() == "}") {
        range.end = range.end.saturating_add(1);
    }
    range
}
