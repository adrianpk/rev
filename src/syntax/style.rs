/// Fixed semantic palette groups supplied by the bundled Rust query.
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
        match name.split('.').next()? {
            "type" | "constructor" => Some(Self::Type),
            "function" => Some(Self::Function),
            "constant" => Some(Self::Number),
            "punctuation" | "operator" => Some(Self::Punctuation),
            _ => None,
        }
    }
}
