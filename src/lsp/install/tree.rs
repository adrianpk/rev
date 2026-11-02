use std::fs;
use std::path::{Path, PathBuf};

struct Accounting {
    root: PathBuf,
    pending: Vec<(PathBuf, usize)>,
    entries: usize,
    total: u64,
    limit: u64,
}

/// Count only the owned tree without following directory links. Finite entry
/// and depth limits also bound pathological trees of empty files/directories.
pub(super) fn bytes(root: &Path, limit: u64) -> Result<u64, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("owned storage root: {e}"))?;
    let mut accounting = Accounting {
        pending: vec![(root.clone(), 0)],
        root,
        entries: 0,
        total: 0,
        limit,
    };
    while let Some((directory, depth)) = accounting.pending.pop() {
        if depth > 64 {
            return Err("owned storage exceeds 64 directory levels".into());
        }
        let listing = fs::read_dir(directory).map_err(|e| format!("inspect owned storage: {e}"))?;
        for entry in listing {
            accounting.entry(&entry.map_err(|e| e.to_string())?.path(), depth)?;
        }
    }
    Ok(accounting.total)
}

impl Accounting {
    fn entry(&mut self, path: &Path, depth: usize) -> Result<(), String> {
        self.entries += 1;
        if self.entries > 65536 {
            return Err("owned storage exceeds 65536 entries".into());
        }
        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if metadata.is_symlink() {
            super::storage::contained(&self.root, path, true)?;
        } else if metadata.is_dir() {
            self.pending.push((path.to_owned(), depth + 1));
        } else if metadata.is_file() {
            self.total = self
                .total
                .checked_add(metadata.len())
                .ok_or("owned storage accounting overflow")?;
        } else {
            return Err(format!(
                "owned storage contains a special file: {}",
                path.display()
            ));
        }
        if self.total > self.limit {
            return Err(format!("owned storage exceeds {} bytes", self.limit));
        }
        Ok(())
    }
}
