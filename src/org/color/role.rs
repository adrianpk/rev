use crate::syntax::Style;

/// Only semantic groups backed by currently recognized source forms belong here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Role {
    Heading,
    Keyword,
    Drawer,
    Property,
    Planning,
    Block,
    Code,
    Comment,
    List,
    Table,
    Formula,
    Emphasis,
    Link,
    Target,
    Footnote,
    Entity,
    Malformed,
}

impl Role {
    pub const ALL: [Self; 17] = [
        Self::Heading,
        Self::Keyword,
        Self::Drawer,
        Self::Property,
        Self::Planning,
        Self::Block,
        Self::Code,
        Self::Comment,
        Self::List,
        Self::Table,
        Self::Formula,
        Self::Emphasis,
        Self::Link,
        Self::Target,
        Self::Footnote,
        Self::Entity,
        Self::Malformed,
    ];

    pub(crate) fn fallback(self) -> Style {
        [
            Style::Function,
            Style::Keyword,
            Style::Type,
            Style::Type,
            Style::Number,
            Style::Keyword,
            Style::String,
            Style::Comment,
            Style::Punctuation,
            Style::Punctuation,
            Style::Number,
            Style::Keyword,
            Style::Function,
            Style::Type,
            Style::Number,
            Style::Type,
            Style::Comment,
        ][self as usize]
    }
}
