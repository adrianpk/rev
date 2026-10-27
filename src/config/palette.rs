use serde::Deserialize;

/// Opaque terminal-independent RGB channels, in red/green/blue order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// Decode exactly six ASCII hexadecimal digits with a leading hash.
    pub fn parse(value: &str) -> Option<Self> {
        let digits = value.strip_prefix('#')?;
        if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        Some(Self(
            u8::from_str_radix(&digits[0..2], 16).ok()?,
            u8::from_str_radix(&digits[2..4], 16).ok()?,
            u8::from_str_radix(&digits[4..6], 16).ok()?,
        ))
    }
}

/// Fixed semantic UI roles with the accepted Tokyo Night night defaults.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    pub background: Rgb,
    pub foreground: Rgb,
    pub cursor: Rgb,
    pub selection: Rgb,
    pub selection_text: Rgb,
    pub search: Rgb,
    pub search_text: Rgb,
    pub status: Rgb,
    pub status_text: Rgb,
    pub status_inactive: Rgb,
    pub border: Rgb,
    pub border_active: Rgb,
    pub tree: Rgb,
    pub tree_selected: Rgb,
    pub tree_selected_text: Rgb,
    pub message: Rgb,
    pub warning: Rgb,
    pub error: Rgb,
    pub syntax: SyntaxPalette,
    /// Native gutter roles; startup parsing is wired with the gutter settings.
    pub gutter: GutterPalette,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            background: Rgb(0x1a, 0x1b, 0x26),
            foreground: Rgb(0xc0, 0xca, 0xf5),
            cursor: Rgb(0xc0, 0xca, 0xf5),
            selection: Rgb(0x28, 0x34, 0x57),
            selection_text: Rgb(0xc0, 0xca, 0xf5),
            search: Rgb(0x3d, 0x59, 0xa1),
            search_text: Rgb(0xc0, 0xca, 0xf5),
            status: Rgb(0x16, 0x16, 0x1e),
            status_text: Rgb(0xa9, 0xb1, 0xd6),
            status_inactive: Rgb(0x1a, 0x1b, 0x26),
            border: Rgb(0x3b, 0x42, 0x61),
            border_active: Rgb(0x7a, 0xa2, 0xf7),
            tree: Rgb(0x16, 0x16, 0x1e),
            tree_selected: Rgb(0x28, 0x34, 0x57),
            tree_selected_text: Rgb(0xc0, 0xca, 0xf5),
            message: Rgb(0xc0, 0xca, 0xf5),
            warning: Rgb(0xe0, 0xaf, 0x68),
            error: Rgb(0xdb, 0x4b, 0x4b),
            syntax: SyntaxPalette::default(),
            gutter: GutterPalette::default(),
        }
    }
}

/// Compiled number-field roles in the same immutable renderer palette snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GutterPalette {
    pub background: Rgb,
    pub foreground: Rgb,
    pub current_line: Rgb,
    pub separator: Rgb,
}

impl Default for GutterPalette {
    fn default() -> Self {
        Self {
            background: Rgb(0x1a, 0x1b, 0x26),
            foreground: Rgb(0x56, 0x5f, 0x89),
            current_line: Rgb(0x7a, 0xa2, 0xf7),
            separator: Rgb(0x3b, 0x42, 0x61),
        }
    }
}

/// Source leaves retain color literals for validation and source diagnostics.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialPalette {
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub cursor: Option<String>,
    pub selection: Option<String>,
    pub selection_text: Option<String>,
    pub search: Option<String>,
    pub search_text: Option<String>,
    pub status: Option<String>,
    pub status_text: Option<String>,
    pub status_inactive: Option<String>,
    pub border: Option<String>,
    pub border_active: Option<String>,
    pub tree: Option<String>,
    pub tree_selected: Option<String>,
    pub tree_selected_text: Option<String>,
    pub message: Option<String>,
    pub warning: Option<String>,
    pub error: Option<String>,
    pub syntax: PartialSyntaxPalette,
}

/// The seven existing syntax groups; capture mapping remains syntax-owned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxPalette {
    pub keyword: Rgb,
    pub string: Rgb,
    pub comment: Rgb,
    pub r#type: Rgb,
    pub function: Rgb,
    pub number: Rgb,
    pub punctuation: Rgb,
}

impl Default for SyntaxPalette {
    fn default() -> Self {
        Self {
            keyword: Rgb(0x9d, 0x7c, 0xd8),
            string: Rgb(0x9e, 0xce, 0x6a),
            comment: Rgb(0x56, 0x5f, 0x89),
            r#type: Rgb(0x2a, 0xc3, 0xde),
            function: Rgb(0x7a, 0xa2, 0xf7),
            number: Rgb(0xff, 0x9e, 0x64),
            punctuation: Rgb(0x89, 0xdd, 0xff),
        }
    }
}

/// Source leaves retain color literals for validation and source diagnostics.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialSyntaxPalette {
    pub keyword: Option<String>,
    pub string: Option<String>,
    pub comment: Option<String>,
    pub r#type: Option<String>,
    pub function: Option<String>,
    pub number: Option<String>,
    pub punctuation: Option<String>,
}
