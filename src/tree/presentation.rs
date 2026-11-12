use std::ffi::OsStr;

use crate::config::{Rgb, TreePalette};

/// Derived visual kind, independent of language support and filesystem identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Root,
    Directory,
    Source,
    Document,
    Configuration,
    File,
    Symlink,
}

impl Kind {
    pub(super) fn file(name: &OsStr) -> Self {
        let bytes = name.as_encoded_bytes();
        if [
            ".editorconfig",
            ".gitignore",
            ".gitattributes",
            "Cargo.lock",
            "Makefile",
            "Dockerfile",
        ]
        .iter()
        .any(|name| bytes.eq_ignore_ascii_case(name.as_bytes()))
        {
            return Self::Configuration;
        }
        let suffix = bytes
            .rsplit(|byte| *byte == b'.')
            .next()
            .unwrap_or_default();
        for (kind, suffixes) in [
            (
                Self::Source,
                &[
                    "rs", "go", "c", "h", "js", "jsx", "ts", "tsx", "html", "htm", "css",
                ][..],
            ),
            (Self::Document, &["md", "markdown", "org", "txt", "rst"][..]),
            (
                Self::Configuration,
                &["toml", "json", "yaml", "yml", "ini", "cfg", "conf", "xml"][..],
            ),
        ] {
            if bytes.contains(&b'.')
                && suffixes
                    .iter()
                    .any(|value| suffix.eq_ignore_ascii_case(value.as_bytes()))
            {
                return kind;
            }
        }
        Self::File
    }

    pub(super) fn symbol(self, nerd_fonts: bool, expanded: bool) -> &'static str {
        if nerd_fonts {
            return self.nerd_symbol(expanded);
        }
        match self {
            Self::Root => "◆",
            Self::Directory => "/",
            Self::Source => "λ",
            Self::Document => "≡",
            Self::Configuration => "=",
            Self::File => "·",
            Self::Symlink => "@",
        }
    }

    fn nerd_symbol(self, expanded: bool) -> &'static str {
        match self {
            Self::Root => "\u{f1b2}",
            Self::Directory if expanded => "\u{f07c}",
            Self::Directory => "\u{f07b}",
            Self::Source => "\u{f121}",
            Self::Document => "\u{f0219}",
            Self::Configuration => "\u{e615}",
            Self::File => "\u{f15b}",
            Self::Symlink => "\u{f481}",
        }
    }

    pub(crate) fn foreground(self, palette: &TreePalette) -> Rgb {
        match self {
            Self::Root => palette.root,
            Self::Directory => palette.directory,
            Self::Source => palette.source,
            Self::Document => palette.document,
            Self::Configuration => palette.configuration,
            Self::File => palette.file,
            Self::Symlink => palette.symlink,
        }
    }
}

/// Both catalogs share checked terminal-cell fields. Final filename sanitizing
/// and clipping stay in the display owner; even a zero-width row is bounded.
pub(super) struct Prefix {
    indent: u16,
    icon: u16,
    name: u16,
}

impl Prefix {
    const FIELD_WIDTH: u16 = 1;
    const WIDTH: u16 = 4;

    pub(super) fn new(depth: usize) -> Result<Self, crate::coordinates::Error> {
        use crate::coordinates::Error;
        let depth = depth.min(usize::from(u16::MAX - (Self::WIDTH + 2)) / 2);
        let indent = u16::try_from(depth)
            .ok()
            .and_then(|depth| depth.checked_mul(2))
            .ok_or(Error::Overflow)?;
        let icon = indent.checked_add(2).ok_or(Error::Overflow)?;
        let name = indent.checked_add(Self::WIDTH).ok_or(Error::Overflow)?;
        Ok(Self { indent, icon, name })
    }

    pub(super) fn render(
        self,
        disclosure: &str,
        icon: &str,
    ) -> Result<String, crate::coordinates::Error> {
        use crate::coordinates::Error;
        use unicode_width::UnicodeWidthStr;
        let disclosure_width = u16::try_from(disclosure.width()).map_err(|_| Error::Overflow)?;
        let icon_width = u16::try_from(icon.width()).map_err(|_| Error::Overflow)?;
        let disclosure_padding = Self::FIELD_WIDTH
            .checked_sub(disclosure_width)
            .ok_or(Error::Overflow)?;
        let icon_padding = self
            .name
            .checked_sub(self.icon)
            .and_then(|field| field.checked_sub(1))
            .and_then(|field| field.checked_sub(icon_width))
            .ok_or(Error::Overflow)?;
        Ok(format!(
            "{}{}{} {}{} ",
            " ".repeat(usize::from(self.indent)),
            disclosure,
            " ".repeat(usize::from(disclosure_padding)),
            icon,
            " ".repeat(usize::from(icon_padding))
        ))
    }
}
