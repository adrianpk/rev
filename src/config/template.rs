//! One compiled configuration asset, shared with the exhaustive reference.

const TEMPLATE: &str = include_str!("../../docs/reference/config.toml");

// A packaged root must fit the same source bound as an ordinary user file.
const _: () = assert!(!TEMPLATE.is_empty() && TEMPLATE.len() <= super::load::SOURCE_BYTES);

/// Complete English factory template. Borrowing it performs no path discovery,
/// allocation, file access or network IO; publication belongs to native startup.
pub const fn default_template() -> &'static str {
    TEMPLATE
}
