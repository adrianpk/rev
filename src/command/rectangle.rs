#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RectangleCommand {
    Mark,
    Copy,
    Kill,
}

impl RectangleCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Mark => "rectangle-mark-mode",
            Self::Copy => "copy-rectangle-as-kill",
            Self::Kill => "kill-rectangle",
        }
    }
}
