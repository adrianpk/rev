use std::path::Path;

use super::{Diagnostic, Snapshot, load, resolve};

/// One preparation outcome: a validated snapshot, or complete built-ins with
/// one diagnostic. The session presents that diagnostic after initial file ingress.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Startup {
    pub snapshot: Snapshot,
    pub diagnostic: Option<Diagnostic>,
}

impl Startup {
    /// Read the sole implicit user root once. Unresolvable XDG locations produce
    /// usable complete defaults with an environment diagnostic, never a CWD read.
    pub fn from_env() -> Self {
        match super::Roots::from_env() {
            Ok(roots) => prepare(&roots.config_file()),
            Err(error) => Self {
                diagnostic: Some(Diagnostic::new(Path::new("environment"), error.to_string())),
                ..Self::default()
            },
        }
    }
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
