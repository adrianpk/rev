//! Typed editing contexts and bounded, independently owned durable sessions.
//!
//! This layer does not open buffers, activate services, or register a directory
//! implicitly. Callers prepare a complete candidate and publish it explicitly.

mod catalog;
mod codec;
mod path;
mod position;
mod store;
mod types;

pub use catalog::{Catalog, Prepared};
pub use path::{FileReference, NativePath};
pub use position::{SourcePosition, Viewport};
pub use store::{Publication, Store};
pub use types::*;

pub const MAX_PROJECTS: usize = 256;
pub const MAX_WORKSPACES: usize = 128;
pub const MAX_ROOTS: usize = 64;
pub const MAX_FILES: usize = 512;
pub const MAX_NAME_BYTES: usize = 256;
pub const MAX_PATH_BYTES: usize = 4096;
pub const MAX_SESSION_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_CATALOG_BYTES: usize = 8 * 1024 * 1024;
pub const FILE_ADMISSION_STEPS: usize = 16;

/// Failures contain scope, never malformed input bytes or source contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid(&'static str),
    Unsupported(&'static str),
    Conflict(&'static str),
    Capacity(&'static str),
    Unavailable(&'static str),
    Busy,
    RevisionExhausted,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(scope) => write!(f, "Invalid context {scope}; input preserved"),
            Self::Unsupported(scope) => write!(f, "Unsupported context {scope}; input preserved"),
            Self::Conflict(scope) => write!(f, "Context conflict: {scope}"),
            Self::Capacity(scope) => write!(f, "Context capacity exceeded: {scope}"),
            Self::Unavailable(operation) => write!(f, "Context storage: {operation}"),
            Self::Busy => f.write_str("Context publication lock is busy"),
            Self::RevisionExhausted => f.write_str("Context revision is exhausted"),
        }
    }
}
impl std::error::Error for Error {}
