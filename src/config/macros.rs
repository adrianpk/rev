use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Macros {
    pub ring_limit: usize,
    pub execute_before_append: bool,
}
impl Default for Macros {
    fn default() -> Self {
        Self {
            ring_limit: 8,
            execute_before_append: true,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialMacros {
    pub ring_limit: Option<i64>,
    pub execute_before_append: Option<bool>,
}
