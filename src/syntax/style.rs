/// Fixed semantic palette groups supplied by bundled language queries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Keyword,
    String,
    Comment,
    Type,
    Function,
    Number,
    Punctuation,
    Org(crate::org::Role),
    Markdown(MarkdownRole),
}

/// Native Markdown roles; parser events never cross the private adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkdownRole {
    Emphasis,
    Link,
    Code,
    Quote,
    List,
    Table,
}

impl MarkdownRole {
    pub(crate) fn color(self, palette: &crate::config::MarkdownPalette) -> crate::config::Rgb {
        match self {
            Self::Emphasis => palette.emphasis,
            Self::Link => palette.link,
            Self::Code => palette.code,
            Self::Quote => palette.quote,
            Self::List => palette.list,
            Self::Table => palette.table,
        }
    }
}

impl Style {
    pub(super) fn named(name: &str) -> Option<Self> {
        match name.split('.').next()? {
            "keyword" => Some(Self::Keyword),
            "string" | "escape" => Some(Self::String),
            "comment" => Some(Self::Comment),
            _ => Self::symbol(name),
        }
    }

    fn symbol(name: &str) -> Option<Self> {
        let name = name.split('.').next()?;
        if ["type", "constructor", "tag", "attribute", "property"].contains(&name) {
            return Some(Self::Type);
        }
        match name {
            "function" => Some(Self::Function),
            "constant" | "number" => Some(Self::Number),
            "punctuation" | "operator" => Some(Self::Punctuation),
            _ => None,
        }
    }
}
