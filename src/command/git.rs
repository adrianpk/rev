#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitCommand {
    Status,
    Commit,
    CommitAmend,
    CommitFixup,
    CommitSubmit,
    CommitCancel,
    Quit,
    Refresh,
    ToggleSection,
    Diff,
    VisitFile,
    Discard,
    Mark,
    Unmark,
    UnmarkAll,
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
            Self::Commit => "git-commit",
            Self::CommitAmend => "git-commit-amend",
            Self::CommitFixup => "git-commit-fixup",
            Self::CommitSubmit => "git-commit-submit",
            Self::CommitCancel => "git-commit-cancel",
            Self::Quit => "git-quit",
            Self::Refresh => "git-refresh",
            Self::ToggleSection => "git-toggle-section",
            Self::Diff => "git-diff",
            Self::VisitFile => "git-visit-file",
            Self::Discard => "git-discard",
            Self::Mark => "git-mark",
            Self::Unmark => "git-unmark",
            Self::UnmarkAll => "git-unmark-all",
            Self::Stage => "git-stage",
            Self::Unstage => "git-unstage",
            Self::Dispatch => "git-dispatch",
            Self::Processes => "git-processes",
            Self::ProcessShow => "git-process-show",
            Self::ProcessCancel => "git-process-cancel",
        }
    }
}
