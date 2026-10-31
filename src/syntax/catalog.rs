use crate::config::Grammar;
use std::borrow::Cow;

pub(super) fn language(grammar: Grammar) -> Option<tree_sitter::Language> {
    Some(match grammar {
        Grammar::Rust => tree_sitter_rust::LANGUAGE.into(),
        Grammar::C => tree_sitter_c::LANGUAGE.into(),
        Grammar::Go => tree_sitter_go::LANGUAGE.into(),
        Grammar::Html => tree_sitter_html::LANGUAGE.into(),
        Grammar::Css => tree_sitter_css::LANGUAGE.into(),
        Grammar::Javascript => tree_sitter_javascript::LANGUAGE.into(),
        Grammar::Typescript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Grammar::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        Grammar::Plain => return None,
    })
}

pub(super) fn highlights(grammar: Grammar) -> Option<Cow<'static, str>> {
    Some(match grammar {
        Grammar::Rust => Cow::Borrowed(tree_sitter_rust::HIGHLIGHTS_QUERY),
        Grammar::C => Cow::Borrowed(tree_sitter_c::HIGHLIGHT_QUERY),
        Grammar::Go => Cow::Borrowed(tree_sitter_go::HIGHLIGHTS_QUERY),
        Grammar::Html => Cow::Borrowed(tree_sitter_html::HIGHLIGHTS_QUERY),
        Grammar::Css => Cow::Borrowed(tree_sitter_css::HIGHLIGHTS_QUERY),
        Grammar::Javascript => Cow::Owned(javascript(false, true)),
        Grammar::Typescript => Cow::Owned(javascript(true, false)),
        Grammar::Tsx => Cow::Owned(javascript(true, true)),
        Grammar::Plain => return None,
    })
}

fn javascript(typed: bool, jsx: bool) -> String {
    let mut query = tree_sitter_javascript::HIGHLIGHT_QUERY.to_owned();
    if typed {
        query.push_str(tree_sitter_typescript::HIGHLIGHTS_QUERY);
    }
    if jsx {
        query.push_str(tree_sitter_javascript::JSX_HIGHLIGHT_QUERY);
    }
    query
}

pub(super) struct Labels {
    pub mode: &'static str,
    pub source_mode: &'static str,
}

pub(super) fn labels(grammar: Grammar) -> Labels {
    let (mode, source_mode) = match grammar {
        Grammar::Rust => ("Rust", "Org Src Rust"),
        Grammar::C => ("C", "Org Src C"),
        Grammar::Go => ("Go", "Org Src Go"),
        Grammar::Html => ("HTML", "Org Src HTML"),
        Grammar::Css => ("CSS", "Org Src CSS"),
        Grammar::Javascript => ("JavaScript", "Org Src JavaScript"),
        Grammar::Typescript => ("TypeScript", "Org Src TypeScript"),
        Grammar::Tsx => ("TSX", "Org Src TSX"),
        Grammar::Plain => ("Text", "Org Src"),
    };
    Labels { mode, source_mode }
}
