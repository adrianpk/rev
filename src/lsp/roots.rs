use std::path::{Path, PathBuf};

/// Resolve only configured markers. An explicitly opened containing tree bounds
/// lookup; an unrelated tree cannot change an individual file's fallback.
pub fn resolve(file: &Path, tree: Option<&Path>, markers: &[String]) -> Result<PathBuf, String> {
    let parent = file
        .parent()
        .ok_or("LSP source has no containing directory")?
        .canonicalize()
        .map_err(|e| format!("LSP source directory: {e}"))?;
    if markers.is_empty() {
        return Ok(parent);
    }
    let boundary = tree
        .and_then(|tree| tree.canonicalize().ok())
        .filter(|tree| parent.starts_with(tree));
    for directory in parent.ancestors() {
        if markers.iter().any(|marker| directory.join(marker).exists()) {
            return Ok(directory.to_path_buf());
        }
        if boundary.as_deref() == Some(directory) {
            break;
        }
    }
    Ok(boundary.unwrap_or(parent))
}

pub(super) fn uri(path: &Path) -> Result<String, String> {
    let path = path.to_str().ok_or("LSP root is not UTF-8")?;
    let mut uri = String::from("file://");
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            uri.push(byte as char);
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    Ok(uri)
}
