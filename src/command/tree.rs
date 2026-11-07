#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeCommand {
    Delete,
    Rename,
    Open,
    CreateFile,
    CreateDirectory,
    Toggle,
    Focus,
    Previous,
    Next,
    Expand,
    Collapse,
    Activate,
    Refresh,
    UpDirectory,
}

impl TreeCommand {
    /// Navigation aliases are interpreted by the tree owner, not source editing.
    pub(crate) fn alias(command: super::Command) -> Option<Self> {
        use super::{Command, Motion, Scroll};
        match command {
            Command::Scroll(Scroll::PreviousLine) => Some(Self::Previous),
            Command::Scroll(Scroll::NextLine) => Some(Self::Next),
            Command::Motion(Motion::ForwardChar) => Some(Self::Expand),
            Command::Motion(Motion::BackwardChar) => Some(Self::Collapse),
            Command::Newline => Some(Self::Open),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Delete => "tree-delete",
            Self::Rename => "tree-rename",
            Self::Open => "tree-open",
            Self::CreateFile => "tree-create-file",
            Self::CreateDirectory => "tree-create-directory",
            Self::Toggle => "toggle-file-tree",
            Self::Focus => "focus-file-tree",
            _ => self.navigation_name(),
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Delete => "Confirm removal, including separate recursive consent.",
            Self::Rename => "Rename one entry and reconcile visited paths.",
            Self::Open => "Open a file or toggle a directory.",
            Self::CreateFile => "Create an empty file at the selected directory.",
            Self::CreateDirectory => "Create a directory at the selected directory.",
            Self::Toggle => "Show or hide the left tree.",
            Self::Focus => "Focus the visible tree.",
            _ => self.navigation_description(),
        }
    }

    fn navigation_name(self) -> &'static str {
        match self {
            Self::Previous => "tree-previous-line",
            Self::Next => "tree-next-line",
            Self::Expand => "tree-expand",
            Self::Collapse => "tree-collapse",
            Self::Activate => "tree-toggle-directory",
            Self::Refresh => "tree-refresh",
            Self::UpDirectory => "tree-up-directory",
            _ => unreachable!("entry commands route before navigation names"),
        }
    }

    fn navigation_description(self) -> &'static str {
        match self {
            Self::Previous => "Select the preceding visible entry.",
            Self::Next => "Select the next visible entry.",
            Self::Expand => "Lazily expand a directory.",
            Self::Collapse => "Collapse or select the parent.",
            Self::Activate => "Toggle a directory; leaves report an error.",
            Self::Refresh => "Reload already opened tree branches.",
            Self::UpDirectory => "Show the parent directory of an unanchored file tree.",
            _ => unreachable!("entry commands route before navigation names"),
        }
    }
}
