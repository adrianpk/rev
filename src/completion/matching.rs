use std::collections::BTreeSet;

use super::{Candidate, Symbol};

pub(super) fn candidates(
    symbols: Vec<Symbol>,
    prefix: &str,
    original: &str,
    byte: usize,
) -> Vec<Candidate> {
    let sensitive = prefix.chars().any(char::is_uppercase);
    let filter = normalized(prefix, sensitive);
    let mut ranked: Vec<_> = symbols
        .into_iter()
        .filter(|symbol| visible(symbol, byte))
        .filter_map(|symbol| {
            let name = normalized(&symbol.name, sensitive);
            let class = rank(&name, &filter)?;
            (symbol.name != original).then_some((
                class,
                symbol.origin != 0,
                symbol.byte.abs_diff(byte),
                symbol,
            ))
        })
        .collect();
    ranked.sort_by(|a, b| {
        (a.0, a.1, a.2, a.3.origin, a.3.byte, &a.3.name, a.3.kind)
            .cmp(&(b.0, b.1, b.2, b.3.origin, b.3.byte, &b.3.name, b.3.kind))
    });
    let mut seen = BTreeSet::new();
    ranked
        .into_iter()
        .filter_map(|(_, _, _, symbol)| {
            if !seen.insert(symbol.name.clone()) {
                return None;
            }
            let label = symbol.name.clone();
            (label.len() <= super::MAX_LABEL).then_some(Candidate {
                label,
                kind: super::Kind::native(symbol.kind),
                detail: symbol.kind,
                text: symbol.name,
                range: None,
                additional: Vec::new(),
                proposal: None,
            })
        })
        .take(super::MAX_CANDIDATES)
        .collect()
}

fn visible(symbol: &Symbol, byte: usize) -> bool {
    let Some(scope) = &symbol.scope else {
        return true;
    };
    symbol.origin == 0 && scope.contains(&byte) && (symbol.kind != "binding" || symbol.byte <= byte)
}

fn normalized(text: &str, sensitive: bool) -> String {
    if sensitive {
        text.to_owned()
    } else {
        text.to_lowercase()
    }
}

pub(crate) fn class(name: &str, prefix: &str) -> Option<u8> {
    let sensitive = prefix.chars().any(char::is_uppercase);
    rank(&normalized(name, sensitive), &normalized(prefix, sensitive))
}

fn rank(name: &str, filter: &str) -> Option<u8> {
    if name.starts_with(filter) {
        return Some(0);
    }
    let mut chars = name.chars();
    filter
        .chars()
        .all(|ch| chars.by_ref().any(|next| next == ch))
        .then_some(1)
}
