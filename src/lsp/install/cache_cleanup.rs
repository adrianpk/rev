//! Explicit removal may clean only marked, idle acquisition directories.
//! Unknown or legacy entries remain untouched, including unmarked job names.
use crate::org::Cancellation;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

pub(super) const MARKER: &str = "rev-cache-owner";
pub(super) const OWNER: &[u8] = b"rev-lsp-acquisition-cache-v1\n";

pub(super) fn prune(root: &Path, cancel: &Cancellation) -> Result<usize, String> {
    let Some(namespace) = namespace(root)? else {
        return Ok(0);
    };
    let mut removed = 0;
    for entry in fs::read_dir(&namespace).map_err(|e| e.to_string())? {
        if cancel.is_cancelled() {
            return Err("cache cleanup cancelled; remaining entries retained".into());
        }
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_name().to_string_lossy().starts_with("job-") && clean(&entry.path())? {
            removed += 1;
        }
    }
    Ok(removed)
}

fn namespace(root: &Path) -> Result<Option<std::path::PathBuf>, String> {
    let namespace = root.join("lsp");
    if !namespace.try_exists().map_err(|e| e.to_string())? {
        return Ok(None);
    }
    super::storage::contained(root, &namespace, true)?;
    let namespace = namespace.canonicalize().map_err(|e| e.to_string())?;
    super::tree::bytes(&namespace, super::CACHE_BYTES)?;
    Ok(Some(namespace))
}

fn clean(path: &Path) -> Result<bool, String> {
    let Some(lease) = idle(path)? else {
        return Ok(false);
    };
    super::tree::bytes(path, super::CACHE_BYTES)?;
    super::lease::same(path, &lease)?;
    erase(path)?;
    Ok(true)
}

fn eligible(path: &Path) -> Result<bool, String> {
    Ok(fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .is_dir()
        && marked(path)?)
}

fn idle(path: &Path) -> Result<Option<File>, String> {
    if !eligible(path)? {
        return Ok(None);
    }
    let lease = File::open(path).map_err(|e| e.to_string())?;
    if lease.try_lock().is_err() {
        return Ok(None);
    }
    super::lease::same(path, &lease)?;
    if !marked(path)? {
        return Err("cache ownership marker changed; preserve it for inspection".into());
    }
    Ok(Some(lease))
}

fn erase(path: &Path) -> Result<(), String> {
    fs::remove_dir_all(path).map_err(|e| format!("owned cache cleanup incomplete: {e}"))?;
    File::open(path.parent().ok_or("cache parent missing")?)
        .and_then(|p| p.sync_all())
        .map_err(|e| format!("cache removed; directory durability unconfirmed: {e}"))
}

fn marked(path: &Path) -> Result<bool, String> {
    let path = path.join(MARKER);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    if !metadata.is_file() || metadata.len() != OWNER.len() as u64 {
        return Ok(false);
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(64).read_to_end(&mut bytes))
        .map_err(|e| e.to_string())?;
    Ok(bytes == OWNER)
}
