#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionCommand {
    AtPoint,
    Mode,
    AutoMode,
    Next,
    Previous,
    Accept,
    Cancel,
}

impl CompletionCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Mode => "completion-mode",
            Self::AutoMode => "completion-auto-mode",
            Self::AtPoint => "completion-at-point",
            Self::Next => "completion-next",
            Self::Previous => "completion-previous",
            Self::Accept => "completion-accept",
            Self::Cancel => "completion-cancel",
        }
    }
}
