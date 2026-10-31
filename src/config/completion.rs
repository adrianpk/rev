use serde::Deserialize;

/// Session defaults. All sizes and delays are validated before projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub enabled: bool,
    pub automatic: bool,
    pub delay_ms: usize,
    pub min_prefix: usize,
    pub in_comments: bool,
    pub in_strings: bool,
    pub max_candidates: usize,
    pub popup_rows: usize,
}
impl Default for Completion {
    fn default() -> Self {
        Self {
            enabled: true,
            automatic: true,
            delay_ms: 150,
            min_prefix: 2,
            in_comments: false,
            in_strings: false,
            max_candidates: 200,
            popup_rows: 10,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialCompletion {
    pub enabled: Option<bool>,
    pub automatic: Option<bool>,
    pub delay_ms: Option<i64>,
    pub min_prefix: Option<i64>,
    pub in_comments: Option<bool>,
    pub in_strings: Option<bool>,
    pub max_candidates: Option<i64>,
    pub popup_rows: Option<i64>,
}
