#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionCommand {
    AtPoint,
    Next,
    Previous,
    Accept,
    Cancel,
}

impl CompletionCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::AtPoint => "completion-at-point",
            Self::Next => "completion-next",
            Self::Previous => "completion-previous",
            Self::Accept => "completion-accept",
            Self::Cancel => "completion-cancel",
        }
    }
}
