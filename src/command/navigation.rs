use crate::search::Direction;

/// A source target, independent of its chord and future operator/selection use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Character(Direction),
    Line(Direction),
    LineStart,
    FirstText,
    LineEnd,
    Word(Word),
    Buffer(Direction),
    Find(Find),
    RepeatFind(bool),
    Paragraph(Direction),
    Pair,
    Page { backward: bool, half: bool },
    Screen(Screen),
    Align(Screen),
    Search(Search),
    Jump(Direction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    NextStart,
    PreviousStart,
    NextEnd,
    NextBigStart,
    PreviousBigStart,
    NextBigEnd,
    PreviousEnd,
    PreviousBigEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Find {
    Forward,
    Backward,
    TillForward,
    TillBackward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Search {
    Forward,
    Backward,
    Repeat,
    Reverse,
    WordForward,
    WordBackward,
}

/// One checked request travels intact through the native deferred-layout FIFO.
/// None retains gg/G's default distinction from an explicit one-based line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Navigation {
    pub target: Target,
    pub count: Option<u32>,
    pub character: Option<char>,
}

impl Navigation {
    pub const fn new(target: Target) -> Self {
        Self {
            target,
            count: None,
            character: None,
        }
    }

    pub fn amount(self) -> usize {
        self.count.unwrap_or(1).max(1) as usize
    }

    pub fn name(self) -> &'static str {
        match self.target {
            Target::Character(Direction::Forward) => "modal-forward-char",
            Target::Character(Direction::Backward) => "modal-backward-char",
            Target::Line(Direction::Forward) => "modal-next-line",
            Target::Line(Direction::Backward) => "modal-previous-line",
            Target::LineStart => "modal-line-start",
            Target::FirstText => "modal-line-first-text",
            Target::LineEnd => "modal-line-end",
            Target::Word(word) => word.name(),
            _ => self.other_name(),
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self.target {
            Target::Character(Direction::Forward) => {
                "Next whole grapheme, including a whole LF/CRLF boundary."
            }
            Target::Character(Direction::Backward) => {
                "Previous whole grapheme, including a whole LF/CRLF boundary."
            }
            Target::Line(Direction::Forward) => {
                "Move to the next logical line, retaining the desired display column."
            }
            Target::Line(Direction::Backward) => {
                "Move to the previous logical line, retaining the desired display column."
            }
            Target::LineStart => "Logical line start.",
            Target::FirstText => "First non-whitespace grapheme of the logical line.",
            Target::LineEnd => {
                "Logical content-end boundary before LF/CRLF. A count selects the corresponding following logical line."
            }
            Target::Word(word) => word.description(),
            _ => self.other_description(),
        }
    }

    fn other_name(self) -> &'static str {
        match self.target {
            Target::Buffer(Direction::Forward) => "modal-buffer-end",
            Target::Buffer(Direction::Backward) => "modal-buffer-start",
            Target::Find(find) => find.name(),
            Target::RepeatFind(false) => "modal-repeat-find",
            Target::RepeatFind(true) => "modal-reverse-find",
            Target::Paragraph(Direction::Forward) => "modal-next-paragraph",
            Target::Paragraph(Direction::Backward) => "modal-previous-paragraph",
            Target::Pair => "modal-matching-delimiter",
            _ => self.view_name(),
        }
    }

    fn other_description(self) -> &'static str {
        match self.target {
            Target::Buffer(Direction::Forward) => {
                "End of buffer; with a count, start of that one-based logical line."
            }
            Target::Buffer(Direction::Backward) => {
                "Start of buffer; with a count, start of that one-based logical line."
            }
            Target::Find(find) => find.description(),
            Target::RepeatFind(false) => {
                "Repeat the last successful find in its original direction; till skips its prior match."
            }
            Target::RepeatFind(true) => {
                "Repeat the last successful find in the opposite direction; retain the original repeat direction."
            }
            Target::Paragraph(Direction::Forward) => {
                "Next empty/whitespace-only logical-line boundary, clamped to EOF."
            }
            Target::Paragraph(Direction::Backward) => {
                "Previous empty/whitespace-only logical-line boundary, clamped to BOF."
            }
            Target::Pair => {
                "Move to the matching parenthesis, bracket or brace on the current logical line."
            }
            _ => self.view_description(),
        }
    }

    fn view_name(self) -> &'static str {
        match self.target {
            Target::Page {
                backward: false,
                half: false,
            } => "modal-page-forward",
            Target::Page {
                backward: true,
                half: false,
            } => "modal-page-backward",
            Target::Page {
                backward: false,
                half: true,
            } => "modal-half-page-forward",
            Target::Page {
                backward: true,
                half: true,
            } => "modal-half-page-backward",
            Target::Screen(Screen::Top) => "modal-screen-top",
            Target::Screen(Screen::Middle) => "modal-screen-middle",
            Target::Screen(Screen::Bottom) => "modal-screen-bottom",
            _ => self.final_name(),
        }
    }

    fn view_description(self) -> &'static str {
        match self.target {
            Target::Page {
                backward: false,
                half: false,
            } => "Full-page forward through native geometry and page overlap.",
            Target::Page {
                backward: true,
                half: false,
            } => "Full-page backward through native geometry and page overlap.",
            Target::Page {
                backward: false,
                half: true,
            } => "Half-page forward.",
            Target::Page {
                backward: true,
                half: true,
            } => "Move backward by half a page.",
            Target::Screen(Screen::Top) => "Top displayed text row; count inward from the top.",
            Target::Screen(Screen::Middle) => {
                "Middle displayed text row, including wrapped fragments and folds."
            }
            Target::Screen(Screen::Bottom) => {
                "Bottom displayed text row; count inward from the bottom."
            }
            _ => self.final_description(),
        }
    }

    fn final_name(self) -> &'static str {
        match self.target {
            Target::Jump(Direction::Backward) => "modal-jump-backward",
            Target::Jump(Direction::Forward) => "modal-jump-forward",

            Target::Align(Screen::Top) => "modal-align-top",
            Target::Align(Screen::Middle) => "modal-align-middle",
            Target::Align(Screen::Bottom) => "modal-align-bottom",
            Target::Search(search) => search.name(),
            _ => unreachable!("source targets delegate before viewport lookup"),
        }
    }

    fn final_description(self) -> &'static str {
        match self.target {
            Target::Jump(Direction::Backward) => {
                "Revisit an older checked location, with an optional count."
            }
            Target::Jump(Direction::Forward) => {
                "Revisit a newer checked location, with an optional count."
            }

            Target::Align(Screen::Top) => "Align point at the viewport top without moving it.",
            Target::Align(Screen::Middle) => {
                "Align point at the viewport middle without moving it."
            }
            Target::Align(Screen::Bottom) => {
                "Align point at the viewport bottom without moving it."
            }
            Target::Search(search) => search.description(),
            _ => unreachable!("source targets delegate before viewport lookup"),
        }
    }
}

impl Word {
    fn name(self) -> &'static str {
        match self {
            Self::NextStart => "modal-next-word-start",
            Self::PreviousStart => "modal-previous-word-start",
            Self::NextEnd => "modal-next-word-end",
            Self::NextBigStart => "modal-next-big-word-start",
            Self::PreviousBigStart => "modal-previous-big-word-start",
            Self::NextBigEnd => "modal-next-big-word-end",
            Self::PreviousEnd => "modal-previous-word-end",
            Self::PreviousBigEnd => "modal-previous-big-word-end",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::NextStart => "Start of the next word or punctuation run, skipping whitespace.",
            Self::PreviousStart => "Previous Unicode word start, skipping punctuation.",
            Self::NextEnd => "Next Unicode word's exclusive end boundary, skipping punctuation.",
            Self::NextBigStart => "Next start of a whitespace-separated WORD.",
            Self::PreviousBigStart => "Previous start of a whitespace-separated WORD.",
            Self::NextBigEnd => "Exclusive end of the next whitespace-separated WORD.",
            Self::PreviousEnd => {
                "Exclusive end of the preceding Unicode word, excluding the run containing the point's predecessor."
            }
            Self::PreviousBigEnd => {
                "Exclusive end of the preceding WORD, excluding the run containing the point's predecessor."
            }
        }
    }
}

impl Find {
    fn name(self) -> &'static str {
        match self {
            Self::Forward => "modal-find-forward",
            Self::Backward => "modal-find-backward",
            Self::TillForward => "modal-till-forward",
            Self::TillBackward => "modal-till-backward",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Forward => {
                "Counted next occurrence of one Unicode scalar within the logical line; land on its grapheme start."
            }
            Self::Backward => {
                "Counted previous occurrence of one Unicode scalar within the logical line; land on its grapheme start."
            }
            Self::TillForward => "Grapheme immediately before the counted next occurrence.",
            Self::TillBackward => "Grapheme immediately after the counted previous occurrence.",
        }
    }

    pub fn backward(self) -> bool {
        matches!(self, Self::Backward | Self::TillBackward)
    }

    pub fn till(self) -> bool {
        matches!(self, Self::TillForward | Self::TillBackward)
    }

    pub fn reversed(self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
            Self::TillForward => Self::TillBackward,
            Self::TillBackward => Self::TillForward,
        }
    }
}

impl Search {
    fn name(self) -> &'static str {
        match self {
            Self::Forward => "modal-search-forward",
            Self::Backward => "modal-search-backward",
            Self::Repeat => "modal-search-repeat",
            Self::Reverse => "modal-search-reverse",
            Self::WordForward => "modal-search-word-forward",
            Self::WordBackward => "modal-search-word-backward",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Forward => "Open the shared forward literal search prompt.",
            Self::Backward => "Open the shared backward literal search prompt.",
            Self::Repeat => "Continue the confirmed search in its original direction.",
            Self::Reverse => "Continue the confirmed search in the opposite direction.",
            Self::WordForward => {
                "Search forward for the Unicode word at point with complete word boundaries."
            }
            Self::WordBackward => {
                "Search backward for the Unicode word at point with complete word boundaries."
            }
        }
    }
}
