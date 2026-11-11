use super::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

// Index exports and source archives retain the filesystem inventory. In a
// checkout, local ignored experiments do not become first-party source owners.
pub(super) fn checkout_sources(root: &Path) -> Result<Option<BTreeSet<PathBuf>>> {
    let observed = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(root)
        .output();
    let Ok(observed) = observed else {
        return Ok(None);
    };
    if !observed.status.success() {
        return Ok(None);
    }
    let top = std::str::from_utf8(&observed.stdout)?.trim_end_matches('\n');
    if Path::new(top).canonicalize()? != root {
        return Ok(None);
    }
    let output = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "*.rs",
        ])
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err("cannot discover checkout-owned Rust sources".into());
    }
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| Ok(root.join(std::str::from_utf8(path)?)))
        .collect::<Result<BTreeSet<_>>>()
        .map(Some)
}

// Ignore rules cannot hide explicit Cargo roots or their source directories.
// A root-level build script does not own the entire checkout.
pub(super) fn admitted_source(
    path: &Path,
    directory: &Path,
    roots: &[PathBuf],
    checkout: Option<&BTreeSet<PathBuf>>,
) -> bool {
    if checkout.is_none_or(|sources| sources.contains(path)) {
        return true;
    }
    roots.iter().any(|root| {
        path == root
            || root
                .parent()
                .is_some_and(|parent| parent != directory && path.starts_with(parent))
    })
}
