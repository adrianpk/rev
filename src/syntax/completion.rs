use std::borrow::Cow;
use std::ops::ControlFlow;
use std::time::Instant;

use ropey::RopeSlice;
use tree_sitter::{Query, QueryCursor, QueryCursorOptions, StreamingIterator};

use crate::buffer::Stamp;
use crate::completion::{MAX_LABEL, MAX_SYMBOLS, Symbol};

use super::{Source, Syntax};

pub(super) fn query(
    grammar: crate::config::Grammar,
    language: &tree_sitter::Language,
) -> Option<Query> {
    use crate::config::Grammar;
    let source = match grammar {
        Grammar::Rust => Cow::Borrowed(include_str!("rust.scm")),
        Grammar::C => Cow::Borrowed(include_str!("c.scm")),
        Grammar::Go => Cow::Borrowed(include_str!("go.scm")),
        Grammar::Html => Cow::Borrowed(include_str!("html.scm")),
        Grammar::Css => Cow::Borrowed(include_str!("css.scm")),
        Grammar::Javascript => Cow::Owned(format!(
            "{}\n{}",
            include_str!("script.scm"),
            include_str!("javascript.scm")
        )),
        Grammar::Typescript | Grammar::Tsx => Cow::Owned(format!(
            "{}\n{}",
            include_str!("script.scm"),
            include_str!("typescript.scm")
        )),
        Grammar::Plain => return None,
    };
    Query::new(language, &source).ok()
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
            if let Some(allowed) = literal_policy(current.kind(), policy) {
                return allowed;
            }
            node = current.parent();
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
        if self.mode(stamp) == "Text" {
            return Err("Native completion is unavailable for this buffer".into());
        }
        let parsed = self.parsed.as_ref().expect("current native parser");
        let tree = parsed.tree.as_ref().expect("current native tree");
        let query = parsed
            .completion
            .as_ref()
            .ok_or("Native completion query is unavailable")?;
        let mut cursor = QueryCursor::new();
        cursor.set_match_limit(4096);
        let mut progress = |_: &tree_sitter::QueryCursorState| expired(deadline, cancel);
        let options = QueryCursorOptions::new().progress_callback(&mut progress);
        let mut captures =
            cursor.captures_with_options(query, tree.root_node(), Source(text), options);
        let mut declarations = Declarations::default();
        while let Some((matched, index)) = captures.next() {
            if stopped(deadline, cancel) {
                return Err("Native completion work or symbol limit reached".into());
            }
            let capture = matched.captures()[*index];
            declarations.push(capture.node, query.capture_names()[capture.index as usize])?;
        }
        if cursor.did_exceed_match_limit() || stopped(deadline, cancel) {
            return Err("Native completion work limit reached".into());
        }
        declarations.finish(text, deadline, cancel)
    }
}

#[derive(Default)]
struct Declarations<'a> {
    names: Vec<(tree_sitter::Node<'a>, &'static str)>,
    scopes: super::scope::Scopes,
}

impl<'a> Declarations<'a> {
    fn push(&mut self, node: tree_sitter::Node<'a>, kind: &str) -> Result<(), String> {
        if kind == "scope" {
            return self.scopes.insert(node);
        }
        let Some(kind) = category(kind) else {
            return Ok(());
        };
        if self.names.len() >= MAX_SYMBOLS {
            return Err("Native completion symbol limit reached".into());
        }
        self.names.push((node, kind));
        Ok(())
    }

    fn finish(
        self,
        text: RopeSlice<'_>,
        deadline: Instant,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<Vec<Symbol>, String> {
        let mut symbols = Vec::new();
        for (node, kind) in self.names {
            if stopped(deadline, cancel) {
                return Err("Native completion work limit reached".into());
            }
            if let Some(symbol) = symbol(node, kind, text, &self.scopes)? {
                symbols.push(symbol);
            }
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

fn symbol(
    node: tree_sitter::Node<'_>,
    kind: &str,
    text: RopeSlice<'_>,
    scopes: &super::scope::Scopes,
) -> Result<Option<Symbol>, String> {
    if node.has_error()
        || node.is_missing()
        || node.parent().is_some_and(|parent| parent.has_error())
    {
        return Ok(None);
    }
    let bytes = node.byte_range();
    let Some(kind) = category(kind) else {
        return Ok(None);
    };
    if bytes.len() + 2 + kind.len() > MAX_LABEL {
        return Err("Native completion label limit reached".into());
    }
    Ok(Some(Symbol {
        name: text.byte_slice(bytes.clone()).to_string(),
        kind,
        byte: bytes.start,
        origin: 0,
        scope: scopes.declaration(node, kind)?,
    }))
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
        _ => return structural_category(kind),
    })
}

fn structural_category(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "tag" => "tag",
        "attribute" => "attribute",
        "selector" => "selector",
        "property" => "property",
        _ => return None,
    })
}

fn literal_policy(kind: &str, policy: &crate::config::Completion) -> Option<bool> {
    if ["line_comment", "block_comment", "comment"].contains(&kind) {
        return Some(policy.in_comments);
    }
    [
        "string_literal",
        "raw_string_literal",
        "char_literal",
        "interpreted_string_literal",
        "rune_literal",
        "system_lib_string",
        "string",
        "template_string",
        "regex",
        "jsx_text",
    ]
    .contains(&kind)
    .then_some(policy.in_strings)
}
