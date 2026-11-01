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
}
