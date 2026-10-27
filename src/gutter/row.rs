/// Semantic number-field roles in the existing frame palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Number,
    CurrentLine,
    Separator,
}

/// One real visible logical row. Empty viewport padding has no NumberRow.
/// The optional label also admits an independently hidden current row later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberRow {
    /// Zero-based logical source row, not a terminal row or display column.
    pub line: usize,
    pub current: bool,
    pub label: Option<usize>,
}

impl NumberRow {
    pub fn absolute(line: usize, lines: usize, current: usize) -> Option<Self> {
        if line >= lines {
            return None;
        }
        Some(Self {
            line,
            current: line == current,
            label: Some(line.checked_add(1)?),
        })
    }

    pub fn role(self) -> Role {
        if self.current {
            Role::CurrentLine
        } else {
            Role::Number
        }
    }
}

/// Both admitted representations occupy one terminal cell. A frontend that
/// cannot display the primary glyph selects ASCII at the output boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Separator {
    #[default]
    Unicode,
    Ascii,
}

impl Separator {
    pub fn text(self) -> &'static str {
        match self {
            Self::Unicode => "│",
            Self::Ascii => "|",
        }
    }
}
