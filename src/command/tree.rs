#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeCommand {
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
