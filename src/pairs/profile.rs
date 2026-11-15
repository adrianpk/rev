use super::Kind;

/// Recognized language policy survives disabled colors and missing grammars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Plain,
    Rust,
    C,
    Go,
    Script,
    Json,
    Toml,
    Css,
    Html,
    Org,
    Markdown,
}

impl Profile {
    pub fn language(id: &str) -> Self {
        const LANGUAGES: [(&str, Profile); 12] = [
            ("rust", Profile::Rust),
            ("c", Profile::C),
            ("go", Profile::Go),
            ("javascript", Profile::Script),
            ("typescript", Profile::Script),
            ("tsx", Profile::Script),
            ("json", Profile::Json),
            ("toml", Profile::Toml),
            ("css", Profile::Css),
            ("html", Profile::Html),
            ("org", Profile::Org),
            ("markdown", Profile::Markdown),
        ];
        LANGUAGES
            .into_iter()
            .find(|(name, _)| *name == id)
            .map_or(Self::Plain, |(_, profile)| profile)
    }

    pub(crate) fn slash_comments(self) -> bool {
        matches!(self, Self::Rust | Self::C | Self::Go | Self::Script)
    }

    pub(crate) fn block_comments(self) -> bool {
        self.slash_comments() || self == Self::Css
    }

    pub(crate) fn prose(self) -> bool {
        matches!(self, Self::Plain | Self::Org | Self::Markdown)
    }

    pub(crate) fn single_line(self, kind: Kind) -> bool {
        kind != Kind::Backtick
            && matches!(
                self,
                Self::C | Self::Go | Self::Script | Self::Json | Self::Toml | Self::Css
            )
    }

    pub(crate) fn continued_line(self) -> bool {
        matches!(self, Self::C | Self::Script | Self::Css)
    }

    pub(crate) fn quote(self, quote: Kind) -> bool {
        match quote {
            Kind::DoubleQuote => true,
            Kind::SingleQuote => matches!(
                self,
                Self::C | Self::Go | Self::Script | Self::Toml | Self::Css | Self::Html
            ),
            Kind::Backtick => self == Self::Go,
            _ => false,
        }
    }
}
