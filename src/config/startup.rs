use std::path::Path;

use super::{Diagnostic, Snapshot, load, resolve};

/// One preparation outcome: a validated snapshot, or complete built-ins with
/// one diagnostic. Rendering that diagnostic belongs to startup integration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Startup {
    pub snapshot: Snapshot,
    pub diagnostic: Option<Diagnostic>,
}

/// Prepare the explicitly selected user root once. A rejected graph cannot
/// contribute any accepted leaves, origins or language overrides to fallback.
/// This function does not install settings into a session or watch source files.
pub fn prepare(root: &Path) -> Startup {
    match load(root).and_then(|sources| resolve(&sources)) {
        Ok(snapshot) => Startup {
            snapshot,
            diagnostic: None,
        },
        Err(diagnostic) => Startup {
            snapshot: Snapshot::default(),
            diagnostic: Some(diagnostic),
        },
    }
}
