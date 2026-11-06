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
    /// Legacy input only. Decode its string shape, then discard its value.
    #[serde(deserialize_with = "discard_marker")]
    pub marker: Option<String>,
    pub max_entries: Option<i64>,
    pub context_chars: Option<i64>,
    pub search_chars: Option<i64>,
}

fn discard_marker<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer)?;
    Ok(None)
}
