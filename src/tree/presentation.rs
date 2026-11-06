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

    pub(super) fn symbol(self) -> &'static str {
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
