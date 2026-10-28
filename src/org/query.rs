//! Bounded, data-only Org tag/property expressions. No source string is executed.
mod compare;
mod parse;
pub mod regexp;
mod scalar;
mod search;
pub use search::SearchQuery;

use std::collections::{BTreeMap, BTreeSet};

/// A temporary metadata projection, supplied by a qualified source observation.
#[derive(Debug, Default)]
pub struct MatchData {
    pub tags: Vec<String>,
    /// Canonical uppercase property names; missing keys remain absent.
    pub properties: BTreeMap<String, String>,
    pub finished: bool,
}
/// Compiled tag/property selection: at most 4096 UTF-8 bytes, 32 nested forms
/// and 512 nodes. Node order permits finite iterative evaluation.
#[derive(Debug, Clone)]
pub struct MatchQuery {
    nodes: Vec<Node>,
    root: usize,
}
#[derive(Debug, Clone)]
enum Node {
    All,
    Tag(String),
    Todo(String),
    Unfinished,
    Compare(Comparison),
    Not(usize),
    And(usize, usize),
    Or(usize, usize),
}
#[derive(Debug, Clone)]
struct Comparison {
    key: String,
    operator: Operator,
    present: bool,
    value: Value,
}
#[derive(Debug, Clone, Copy)]
enum Operator {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
#[derive(Debug, Clone)]
enum Value {
    Number(f64),
    Text(String),
}

impl MatchQuery {
    pub fn parse(source: &str) -> Result<Self, String> {
        Self::bounded(source, 4096, 32)
    }
    pub fn bounded(source: &str, bytes: usize, depth: usize) -> Result<Self, String> {
        if !(1..=65536).contains(&bytes) || !(1..=128).contains(&depth) {
            return Err("Org query requires bytes 1..65536 and depth 1..128".into());
        }
        if source.is_empty() || source.len() > bytes || source.chars().any(char::is_control) {
            return Err("Org match requires 1..4096 UTF-8 bytes without control characters".into());
        }
        parse::Parser::new(source, depth).query()
    }
    /// Only properties named by the compiled query need source projection.
    pub fn properties(&self) -> BTreeSet<&str> {
        self.nodes
            .iter()
            .filter_map(|node| match node {
                Node::Compare(c) => Some(c.key.as_str()),
                Node::Todo(_) | Node::Unfinished => Some("TODO"),
                _ => None,
            })
            .collect()
    }
    pub fn matches(&self, data: &MatchData) -> Result<bool, String> {
        self.matches_with_units(data, &crate::config::DurationUnits::default())
    }
    pub fn matches_with_units(
        &self,
        data: &MatchData,
        units: &crate::config::DurationUnits,
    ) -> Result<bool, String> {
        data.admit()?;
        let tags: BTreeSet<_> = data.tags.iter().map(String::as_str).collect();
        let mut values = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            values.push(evaluate(node, &values, data, &tags, units));
        }
        Ok(values[self.root])
    }
}
fn evaluate(
    node: &Node,
    values: &[bool],
    data: &MatchData,
    tags: &BTreeSet<&str>,
    units: &crate::config::DurationUnits,
) -> bool {
    match node {
        Node::All => true,
        Node::Tag(tag) => tags.contains(tag.as_str()),
        Node::Todo(todo) => data.properties.get("TODO") == Some(todo),
        Node::Unfinished => data.unfinished(),
        Node::Compare(comparison) => {
            comparison.matches(data.properties.get(&comparison.key), units)
        }
        Node::Not(node) => !values[*node],
        Node::And(left, right) => values[*left] && values[*right],
        Node::Or(left, right) => values[*left] || values[*right],
    }
}

impl MatchData {
    fn unfinished(&self) -> bool {
        !self.finished && self.properties.get("TODO").is_some_and(|v| !v.is_empty())
    }
    fn admit(&self) -> Result<(), String> {
        if self.tags.len() > 8192 || self.properties.len() > 512 {
            return Err("Org match metadata exceeds record limits".into());
        }
        let bytes: usize = self
            .tags
            .iter()
            .map(String::len)
            .chain(
                self.properties
                    .iter()
                    .map(|(k, v)| k.len().saturating_add(v.len())),
            )
            .try_fold(0usize, |a, b| a.checked_add(b))
            .ok_or("Org match metadata size overflow")?;
        if bytes > 1048576 {
            return Err("Org match metadata exceeds 1048576 UTF-8 bytes".into());
        }
        Ok(())
    }
}

pub(super) fn task_keyword(word: &str) -> bool {
    !word.is_empty()
        && word.len() <= 128
        && !word
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "|()".contains(c))
}
