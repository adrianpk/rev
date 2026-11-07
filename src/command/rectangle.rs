#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RectangleCommand {
    Mark,
    Copy,
    Kill,
    Yank,
}

impl RectangleCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Mark => "rectangle-mark-mode",
            Self::Copy => "copy-rectangle-as-kill",
            Self::Kill => "kill-rectangle",
            Self::Yank => "yank-rectangle",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Mark => "Toggle display-column selection.",
            Self::Copy => "Retain selected rectangle rows.",
            Self::Kill => "Remove and retain the rectangle as one edit.",
            Self::Yank => "Insert retained rows with required padding.",
        }
    }
}
