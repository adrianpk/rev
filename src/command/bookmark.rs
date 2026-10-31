#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BookmarkCommand {
    Set,
    SetNoOverwrite,
    Jump,
    Delete,
}

impl BookmarkCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Set => "bookmark-set",
            Self::SetNoOverwrite => "bookmark-set-no-overwrite",
            Self::Jump => "bookmark-jump",
            Self::Delete => "bookmark-delete",
        }
    }
}
