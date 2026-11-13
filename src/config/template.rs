//! Separate compiled assets for inherited startup and the exhaustive reference.

const TEMPLATE: &str = include_str!("../../docs/reference/config.toml");
const STARTUP: &str = include_str!("../../docs/reference/startup.toml");

// A packaged root must fit the same source bound as an ordinary user file.
const _: () = assert!(!TEMPLATE.is_empty() && TEMPLATE.len() <= super::load::SOURCE_BYTES);
const _: () = assert!(!STARTUP.is_empty() && STARTUP.len() <= super::load::SOURCE_BYTES);

/// Complete English factory template. Borrowing it performs no path discovery,
/// allocation, file access or network IO; publication belongs to native startup.
pub const fn default_template() -> &'static str {
    TEMPLATE
}

/// Commented startup examples inherit current defaults until explicitly enabled.
pub const fn startup_template() -> &'static str {
    STARTUP
}
