#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BookmarkCommand {
    Set,
    SetNoOverwrite,
    Jump,
    Delete,
    Save,
    Reload,
    List,
    Next,
    Previous,
    Markers,
    MenuDelete,
    MenuUnmark,
    MenuExecute,
    MenuVisit,
}

impl BookmarkCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Set => "bookmark-set",
            Self::SetNoOverwrite => "bookmark-set-no-overwrite",
            Self::Jump => "bookmark-jump",
            Self::Delete => "bookmark-delete",
            Self::Save => "bookmark-save",
            Self::Reload => "bookmark-reload",
            Self::List => "list-bookmarks",
            Self::Next => "bookmark-next",
            Self::Previous => "bookmark-previous",
            Self::Markers => "bookmark-markers-mode",
            Self::MenuDelete => "bookmark-bmenu-delete",
            Self::MenuUnmark => "bookmark-bmenu-unmark",
            Self::MenuExecute => "bookmark-bmenu-execute-deletions",
            Self::MenuVisit => "bookmark-bmenu-this-window",
        }
    }
}
