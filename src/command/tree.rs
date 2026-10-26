#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeCommand {
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
}

impl TreeCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Open => "tree-open",
            Self::CreateFile => "tree-create-file",
            Self::CreateDirectory => "tree-create-directory",
            Self::Toggle => "toggle-file-tree",
            Self::Focus => "focus-file-tree",
            Self::Previous => "tree-previous-line",
            Self::Next => "tree-next-line",
            Self::Expand => "tree-expand",
            Self::Collapse => "tree-collapse",
            Self::Activate => "tree-toggle-directory",
            Self::Refresh => "tree-refresh",
        }
    }
}
