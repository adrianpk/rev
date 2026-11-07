#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scroll {
    NextLine,
    PreviousLine,
    PageForward,
    PageBackward,
    Recenter,
}

impl Scroll {
    pub fn name(self) -> &'static str {
        match self {
            Self::NextLine => "next-line",
            Self::PreviousLine => "previous-line",
            Self::PageForward => "scroll-up-command",
            Self::PageBackward => "scroll-down-command",
            Self::Recenter => "recenter-top-bottom",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::NextLine => {
                "Move down one visual row when wrapped, otherwise one logical row; retain the intended display column."
            }
            Self::PreviousLine => {
                "Move up one visual row when wrapped, otherwise one logical row; retain the intended display column."
            }
            Self::PageForward => "Advance one page.",
            Self::PageBackward => "Move back one page.",
            Self::Recenter => "Cycle point's line through center/top/bottom.",
        }
    }

    pub(crate) fn backward(self) -> bool {
        matches!(self, Self::PreviousLine | Self::PageBackward)
    }

    pub(crate) fn page(self) -> bool {
        matches!(self, Self::PageForward | Self::PageBackward)
    }
}
