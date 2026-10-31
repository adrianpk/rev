use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BookmarkSave {
    #[default]
    OnExit,
    OnChange,
    Manual,
}

/// User-wide storage and bounded source evidence. Unset storage is resolved
/// only by executable startup; embedded sessions retain explicit host ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bookmarks {
    pub file: Option<PathBuf>,
    pub save: BookmarkSave,
    pub markers: bool,
    pub marker: String,
    pub max_entries: usize,
    pub context_chars: usize,
    pub search_chars: usize,
}
impl Default for Bookmarks {
    fn default() -> Self {
        Self {
            file: None,
            save: BookmarkSave::OnExit,
            markers: true,
            marker: "●".into(),
            max_entries: 1024,
            context_chars: 64,
            search_chars: 262144,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialBookmarks {
    pub file: Option<PathBuf>,
    pub save: Option<BookmarkSave>,
    pub markers: Option<bool>,
    pub marker: Option<String>,
    pub max_entries: Option<i64>,
    pub context_chars: Option<i64>,
    pub search_chars: Option<i64>,
}
