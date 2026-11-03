//! Fresh native observations at submission; never a relevance or index owner.
use crate::{buffer::BufferId, session::Session};
use serde_json::{Value, json};
use std::path::Path;

impl Session {
    pub(super) fn ai_sources(&self, root: &Path, current: BufferId) -> Result<Vec<Value>, String> {
        let private = crate::ai::pi::private_roots()?;
        if private.iter().any(|path| root.starts_with(path)) {
            return Err("Pi directory root is private application state".into());
        }
        let mut snapshots = Vec::new();
        let mut snapshot_bytes = 0usize;
        for source in self.buffers.iter() {
            let Some(file) = source.file() else {
                continue;
            };
            let Ok(path) = file.path().strip_prefix(root) else {
                continue;
            };
            if !crate::ai::pi::source_path(path)
                || private.iter().any(|p| file.path().starts_with(p))
            {
                if source.id() == current || source.is_modified() {
                    return Err(
                        "Pi source snapshot is excluded by credential/internal-state policy".into(),
                    );
                }
                continue;
            }
            if source.id() != current && !source.is_modified() {
                continue;
            }
            if snapshots.len() == 16 || source.len_bytes() > 1_048_576 {
                return Err("Pi native source map exceeds 16 buffers or 1 MiB per source".into());
            }
            snapshot_bytes = snapshot_bytes.saturating_add(source.len_bytes());
            if snapshot_bytes > self.buffers.snapshot().settings().ai.pi.max_input_bytes {
                return Err("Pi native sources exceed max_input_bytes".into());
            }
            use std::os::unix::fs::MetadataExt;
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
            snapshots.push(json!({
                "path":path, "buffer_id":source.id().to_string(), "revision":source.stamp().revision.to_string(),
                "complete":true, "dirty":source.is_modified(), "bom":source.has_bom(),
                "text":text
            }));
        }
        Ok(snapshots)
    }
}
