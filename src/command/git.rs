#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitCommand {
    Status,
    Quit,
    Refresh,
    ToggleSection,
    Diff,
    VisitFile,
    Stage,
    Unstage,
    Dispatch,
    Processes,
    ProcessShow,
    ProcessCancel,
}

impl GitCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Status => "git-status",
            Self::Quit => "git-quit",
            Self::Refresh => "git-refresh",
            Self::ToggleSection => "git-toggle-section",
            Self::Diff => "git-diff",
            Self::VisitFile => "git-visit-file",
            Self::Stage => "git-stage",
            Self::Unstage => "git-unstage",
            Self::Dispatch => "git-dispatch",
            Self::Processes => "git-processes",
            Self::ProcessShow => "git-process-show",
            Self::ProcessCancel => "git-process-cancel",
        }
    }
}
