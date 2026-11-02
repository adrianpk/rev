//! Explicit, optional managed installation. Registry reads never start work;
//! mutations own one native job slot and OS locks for their entire lifetime.
mod cache;
mod evidence;
mod go;
mod helper;
mod package;
mod probe;
mod process;
mod progress;
mod record;
mod rust;
mod storage;
mod tree;
mod worker;

pub use go::install_go;
pub use progress::Phase;
pub use record::{Installation, Launch, Registry, State};
pub use rust::install_rust;
pub use storage::{Candidate, Storage};
pub use worker::{Manager, Outcome};

pub const RECORDS: usize = 32;
pub const REGISTRY_BYTES: usize = 2 * 1024 * 1024;
pub const ARTIFACT_BYTES: u64 = 256 * 1024 * 1024;
pub const PAYLOAD_BYTES: u64 = 1024 * 1024 * 1024;
pub const CACHE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const OUTPUT_BYTES: usize = 1024 * 1024;
pub const DISPLAY_BYTES: usize = 64 * 1024;
pub const OPERATION_SECONDS: u64 = 900;
