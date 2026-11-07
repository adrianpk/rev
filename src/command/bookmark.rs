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
            _ => self.navigation_name(),
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Set => "Name or replace the current file position.",
            Self::SetNoOverwrite => "Name the current position without replacement.",
            Self::Jump => "Complete a name and visit its checked location.",
            Self::Delete => "Complete a name and remove that bookmark.",
            Self::Save => "Checkpoint the user collection without saving sources.",
            Self::Reload => "Validate disk and confirm replacing unsaved bookmarks.",
            Self::List => "Open the read-only native bookmark list.",
            _ => self.navigation_description(),
        }
    }
    fn navigation_name(self) -> &'static str {
        match self {
            Self::Next => "bookmark-next",
            Self::Previous => "bookmark-previous",
            Self::Markers => "bookmark-markers-mode",
            Self::MenuDelete => "bookmark-bmenu-delete",
            Self::MenuUnmark => "bookmark-bmenu-unmark",
            Self::MenuExecute => "bookmark-bmenu-execute-deletions",
            Self::MenuVisit => "bookmark-bmenu-this-window",
            _ => unreachable!("bookmark navigation command"),
        }
    }

    fn navigation_description(self) -> &'static str {
        match self {
            Self::Next => {
                "Visit the next resolved current-file record, wrapping and stepping same-position names."
            }
            Self::Previous => {
                "Visit the previous resolved current-file record in source/name order."
            }
            Self::Markers => "Toggle marker visibility for the selected editing view.",
            Self::MenuDelete => "Mark the selected record for deletion and advance.",
            Self::MenuUnmark => "Unmark the selected record and advance.",
            Self::MenuExecute => "Execute checked marked deletions.",
            Self::MenuVisit => "Visit the selected stable record in this pane.",
            _ => unreachable!("bookmark navigation command"),
        }
    }
}
