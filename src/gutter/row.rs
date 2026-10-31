use super::{CurrentLine, Mode, Settings};

/// Semantic gutter roles in the existing frame palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Number,
    CurrentLine,
    Separator,
    Bookmark,
}

/// One real visible logical row. Empty viewport padding has no NumberRow.
/// A hidden current-line form retains its row identity and semantic role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberRow {
    /// Zero-based logical source row, not a terminal row or display column.
    pub line: usize,
    pub current: bool,
    pub label: Option<usize>,
}

impl NumberRow {
    pub fn new(line: usize, lines: usize, current: usize, settings: Settings) -> Option<Self> {
        if line >= lines || current >= lines {
            return None;
        }
        Some(Self {
            line,
            current: line == current,
            label: label(line, current, settings),
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

fn label(line: usize, current: usize, settings: Settings) -> Option<usize> {
    if line == current {
        return match settings.current_line {
            CurrentLine::Absolute => line.checked_add(1),
            CurrentLine::Zero => Some(0),
            CurrentLine::Hidden => None,
        };
    }
    match settings.mode {
        Mode::Absolute => line.checked_add(1),
        Mode::Relative => Some(line.abs_diff(current)),
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
