//! Fresh native observations at submission; never a relevance or index owner.
use crate::{
    buffer::{Buffer, BufferId},
    session::Session,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

struct NativeMap {
    snapshots: Vec<Value>,
    bytes: usize,
    cap: usize,
}

impl NativeMap {
    fn push(&mut self, source: &Buffer, path: &Path) -> Result<(), String> {
        if self.snapshots.len() == 16 || source.len_bytes() > 1_048_576 {
            return Err("Pi native source map exceeds 16 buffers or 1 MiB per source".into());
        }
        self.bytes = self.bytes.saturating_add(source.len_bytes());
        if self.bytes > self.cap {
            return Err("Pi native sources exceed max_input_bytes".into());
        }
        self.snapshots.push(snapshot(source, path)?);
        Ok(())
    }
}

impl Session {
    pub(super) fn ai_sources(&self, root: &Path, current: BufferId) -> Result<Vec<Value>, String> {
        let private = crate::ai::pi::private_roots()?;
        if private.iter().any(|path| root.starts_with(path)) {
            return Err("Pi directory root is private application state".into());
        }
        let mut map = NativeMap {
            snapshots: Vec::new(),
            bytes: 0,
            cap: self.buffers.snapshot().settings().ai.pi.max_input_bytes,
        };
        for source in self.buffers.iter() {
            if let Some(path) = selected(source, root, &private, current)? {
                map.push(source, path)?;
            }
        }
        Ok(map.snapshots)
    }
}

fn selected<'a>(
    source: &'a Buffer,
    root: &Path,
    private: &[PathBuf],
    current: BufferId,
) -> Result<Option<&'a Path>, String> {
    let Some(file) = source.file() else {
        return Ok(None);
    };
    let Ok(path) = file.path().strip_prefix(root) else {
        return Ok(None);
    };
    if !crate::ai::pi::source_path(path) || private.iter().any(|p| file.path().starts_with(p)) {
        if source.id() == current || source.is_modified() {
            return Err(
                "Pi source snapshot is excluded by credential/internal-state policy".into(),
            );
        }
        return Ok(None);
    }
    if source.id() != current && !source.is_modified() {
        return Ok(None);
    }
    Ok(Some(path))
}

fn snapshot(source: &Buffer, path: &Path) -> Result<Value, String> {
    use std::os::unix::fs::MetadataExt;
    let file = source.file().expect("selected native source association");
    if let Ok(meta) = std::fs::symlink_metadata(file.path())
        && (!meta.is_file() || meta.nlink() != 1)
    {
        return Err("Pi native source association is a link or unsupported file".into());
    }
    let span = source
        .span(0..source.len_chars())
        .map_err(|e| e.to_string())?;
    let text = source.slice(&span).map_err(|e| e.to_string())?.to_string();
    if text.contains('\0') {
        return Err("Pi native source contains NUL".into());
    }
    Ok(json!({
        "path":path, "buffer_id":source.id().to_string(), "revision":source.stamp().revision.to_string(),
        "complete":true, "dirty":source.is_modified(), "bom":source.has_bom(), "text":text
    }))
}
