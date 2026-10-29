//! Only this private directory and its contents belong to this compiler job.
use crate::org::{Cancellation, export::document};
use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub(super) struct Scratch {
    root: PathBuf,
    owned: bool,
}
impl Scratch {
    pub fn new() -> Result<Self, String> {
        for _ in 0..128 {
            let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
            let root =
                std::env::temp_dir().join(format!("rev-org-pdf-{}-{sequence}", std::process::id()));
            match DirBuilder::new().mode(0o700).create(&root) {
                Ok(()) => return Ok(Self { root, owned: true }),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(format!("Create private PDF workspace: {e}")),
            }
        }
        Err("Cannot allocate private PDF workspace".into())
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn substitute(&self, token: String) -> Result<String, String> {
        let path = match token.as_str() {
            "{input}" => self.root.join("document.tex"),
            "{output}" => self.root.join("document.pdf"),
            "{output_dir}" => self.root.clone(),
            _ => return Ok(token),
        };
        Ok(path
            .to_str()
            .ok_or("PDF workspace path is not UTF-8")?
            .into())
    }
    pub fn input(&self, text: &str, cancel: &Cancellation) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(self.root.join("document.tex"))
            .map_err(|e| format!("Stage TeX input: {e}"))?;
        for bytes in text.as_bytes().chunks(8192) {
            document::check(cancel)?;
            file.write_all(bytes)
                .map_err(|e| format!("Write TeX input: {e}"))?;
        }
        Ok(())
    }
    pub fn output(&self, limit: usize, cancel: &Cancellation) -> Result<Vec<u8>, String> {
        document::check(cancel)?;
        let path = self.root.join("document.pdf");
        let before = fs::symlink_metadata(&path).map_err(|e| format!("Read compiler PDF: {e}"))?;
        if !before.file_type().is_file() || before.len() > limit as u64 {
            return Err("Compiler PDF must be a bounded regular file".into());
        }
        let bytes = read(&path, limit, cancel)?;
        let after = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        unchanged(&before, &after)?;
        envelope(&bytes)?;
        Ok(bytes)
    }
    pub fn clean(&mut self) -> Result<(), String> {
        fs::remove_dir_all(&self.root).map_err(|e| format!("Remove owned PDF workspace: {e}"))?;
        self.owned = false;
        Ok(())
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        if self.owned {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
fn read(path: &Path, limit: usize, cancel: &Cancellation) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut output = Vec::new();
    loop {
        document::check(cancel)?;
        let mut chunk = [0; 8192];
        let count = file.read(&mut chunk).map_err(|e| e.to_string())?;
        if count == 0 {
            return Ok(output);
        }
        if count > limit.saturating_sub(output.len()) {
            return Err("Compiler PDF exceeds output byte limit".into());
        }
        output.extend_from_slice(&chunk[..count]);
    }
}

fn unchanged(before: &fs::Metadata, after: &fs::Metadata) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;
    let identity = |m: &fs::Metadata| (m.dev(), m.ino(), m.len(), m.mtime(), m.mtime_nsec());
    if !after.file_type().is_file() || identity(before) != identity(after) {
        return Err("Compiler PDF changed while reading".into());
    }
    Ok(())
}
fn envelope(bytes: &[u8]) -> Result<(), String> {
    if !bytes.starts_with(b"%PDF-") || !bytes.windows(5).any(|w| w == b"%%EOF") {
        return Err("Compiler output has no PDF envelope".into());
    }
    Ok(())
}
