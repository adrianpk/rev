use crate::buffer::{Buffer, edit::Point};
use crate::config::Grammar;

use super::Symbol;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Context {
    #[default]
    Names,
    Tag,
    Attribute,
    Selector,
    Property,
    CustomProperty,
    Unsupported,
}

impl Context {
    pub fn accepts(self, symbol: &Symbol) -> bool {
        match self {
            Self::Names => !["tag", "attribute", "selector", "property"].contains(&symbol.kind),
            Self::Tag => symbol.kind == "tag",
            Self::Attribute => symbol.kind == "attribute",
            Self::Selector => symbol.kind == "selector",
            Self::Property => symbol.kind == "property",
            Self::CustomProperty => symbol.kind == "property" && symbol.name.starts_with("--"),
            Self::Unsupported => false,
        }
    }

    pub fn catalog(self) -> Vec<Symbol> {
        let (names, kind) = match self {
            Self::Tag => (super::vocabulary::TAGS, "tag"),
            Self::Attribute => (super::vocabulary::ATTRIBUTES, "attribute"),
            Self::Property => (super::vocabulary::PROPERTIES, "property"),
            _ => return Vec::new(),
        };
        names
            .iter()
            .map(|name| Symbol {
                name: (*name).to_owned(),
                kind,
                byte: usize::MAX,
                origin: usize::MAX,
                scope: None,
            })
            .collect()
    }
}

pub(super) fn trigger(buffer: &Buffer, point: Point) -> bool {
    let end = point.position.0;
    let Ok(span) = buffer.span(end.saturating_sub(2)..end) else {
        return false;
    };
    let Ok(text) = buffer.slice(&span) else {
        return false;
    };
    let text = text.to_string();
    match buffer.completion_grammar() {
        Grammar::Html => {
            text.ends_with(['<', '/']) || text.chars().last().is_some_and(char::is_whitespace)
        }
        Grammar::Css => text.ends_with(['.', '#', '{', ';', '(']),
        _ => {
            text.ends_with('.')
                || text.ends_with("::")
                || (buffer.completion_grammar() == Grammar::C && text.ends_with("->"))
        }
    }
}
