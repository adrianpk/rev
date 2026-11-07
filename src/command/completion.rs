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

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Mode => "Toggle completion for this session.",
            Self::AutoMode => "Toggle automatic requests while retaining explicit invocation.",
            Self::AtPoint => "Show completion suggestions at point.",
            Self::Next => "Select the next completion candidate.",
            Self::Previous => "Select the previous completion candidate.",
            Self::Accept => "Apply the selected completion as one undoable edit.",
            Self::Cancel => "Dismiss the popup without editing source.",
        }
    }
}
