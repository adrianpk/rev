#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitCommand {
    Status,
    Quit,
    Refresh,
    ToggleSection,
    Dispatch,
}

impl GitCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Status => "git-status",
            Self::Quit => "git-quit",
            Self::Refresh => "git-refresh",
            Self::ToggleSection => "git-toggle-section",
            Self::Dispatch => "git-dispatch",
        }
    }
}
