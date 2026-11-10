use super::*;
use std::{fs, io::Read, os::unix::fs::MetadataExt, path::Component};

/// An ordinary, bounded saved file retains its identity through one capture.
struct SavedFile {
    path: PathBuf,
    metadata: fs::Metadata,
    cap: usize,
    length: usize,
}

pub(super) fn capture(
    conflict: &Conflict,
    cap: usize,
    admission: &mut Lease,
) -> Result<Option<SavedResult>, String> {
    let Some(file) = SavedFile::locate(conflict, cap)? else {
        return Ok(None);
    };
    admission.grow(file.length + 4096)?;
    file.read().map(Some)
}

fn saved_path(conflict: &Conflict) -> Result<PathBuf, String> {
    let root = conflict
        .sequence
        .refs
        .observation
        .identity
        .worktree
        .as_ref()
        .ok_or("Conflict result requires a working tree")?;
    let mut parent = root.clone();
    for component in conflict.change.path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err("Conflict path is not an exact relative worktree path".into());
        }
        let m = fs::symlink_metadata(&parent).map_err(|e| format!("Conflict parent: {e}"))?;
        if !m.is_dir() || m.file_type().is_symlink() {
            return Err("Conflict parent is not an ordinary canonical directory".into());
        }
        parent.push(component);
    }
    Ok(parent)
}

impl SavedFile {
    fn locate(conflict: &Conflict, cap: usize) -> Result<Option<Self>, String> {
        let path = saved_path(conflict)?;
        let metadata = match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("Conflict saved result: {e}")),
            Ok(m) => m,
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > cap as u64 {
            return Err("Saved conflict result is not an admitted ordinary complete file".into());
        }
        let length = usize::try_from(metadata.len()).expect("file length admitted below usize cap");
        Ok(Some(Self {
            path,
            metadata,
            cap,
            length,
        }))
    }

    fn open(&self) -> Result<fs::File, String> {
        let descriptor = rustix::fs::open(
            &self.path,
            rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::NOFOLLOW
                | rustix::fs::OFlags::NONBLOCK,
            rustix::fs::Mode::empty(),
        )
        .map_err(|e| e.to_string())?;
        let file = fs::File::from(descriptor);
        let opened = file.metadata().map_err(|e| e.to_string())?;
        if fingerprint(&self.metadata) != fingerprint(&opened) {
            return Err("Saved conflict file identity changed during open".into());
        }
        Ok(file)
    }

    fn read(&self) -> Result<SavedResult, String> {
        let file = self.open()?;
        let mut bytes = Vec::with_capacity(self.length);
        file.take(self.cap as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > self.cap || bytes.contains(&0) || std::str::from_utf8(&bytes).is_err() {
            return Err("Saved resolution is partial, binary or non-text".into());
        }
        let after = fs::symlink_metadata(&self.path).map_err(|e| e.to_string())?;
        if bytes.len() as u64 != self.metadata.len()
            || fingerprint(&self.metadata) != fingerprint(&after)
        {
            return Err("Saved conflict result changed during capture".into());
        }
        Ok(SavedResult {
            bytes,
            mode: if self.metadata.mode() & 0o111 == 0 {
                0o100644
            } else {
                0o100755
            },
            fingerprint: fingerprint(&self.metadata),
        })
    }
}
