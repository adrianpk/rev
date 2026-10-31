use crate::config::Grammar;

pub(super) fn language(grammar: Grammar) -> Option<tree_sitter::Language> {
    Some(match grammar {
        Grammar::Rust => tree_sitter_rust::LANGUAGE.into(),
        Grammar::C => tree_sitter_c::LANGUAGE.into(),
        Grammar::Go => tree_sitter_go::LANGUAGE.into(),
        Grammar::Plain => return None,
    })
}

pub(super) fn highlights(grammar: Grammar) -> Option<&'static str> {
    Some(match grammar {
        Grammar::Rust => tree_sitter_rust::HIGHLIGHTS_QUERY,
        Grammar::C => tree_sitter_c::HIGHLIGHT_QUERY,
        Grammar::Go => tree_sitter_go::HIGHLIGHTS_QUERY,
        Grammar::Plain => return None,
    })
}

pub(super) fn name(grammar: Grammar) -> &'static str {
    match grammar {
        Grammar::Rust => "Rust",
        Grammar::C => "C",
        Grammar::Go => "Go",
        Grammar::Plain => "Text",
    }
}
