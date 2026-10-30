//! Only newly created private workspace contents belong to this operation.
use crate::org::Cancellation;
use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    io::Write,
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
            let root = std::env::temp_dir().join(format!(
                "rev-org-execution-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match DirBuilder::new().mode(0o700).create(&root) {
                Ok(()) => {
                    let s = Self { root, owned: true };
                    s.directories()?;
                    return Ok(s);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(format!("Create source workspace: {e}")),
            }
        }
        Err("Cannot allocate private source workspace".into())
    }
    fn directories(&self) -> Result<(), String> {
        for name in ["home", "config", "cache", "data", "state", "runtime", "tmp"] {
            DirBuilder::new()
                .mode(0o700)
                .create(self.root.join(name))
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn substitute(&self, token: &str) -> Result<String, String> {
        let path = match token {
            "{input}" => self.root.join("source.code"),
            "{output}" => self.root.join("program"),
            "{workspace}" => self.root.clone(),
            _ => return Ok(token.into()),
        };
        path.to_str()
            .map(str::to_owned)
            .ok_or("Source workspace path is not UTF-8".into())
    }
    pub fn input(&self, source: &str, cancel: &Cancellation) -> Result<(), String> {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(self.root.join("source.code"))
            .map_err(|e| e.to_string())?;
        for chunk in source.as_bytes().chunks(8192) {
            super::check(cancel)?;
            f.write_all(chunk)
                .map_err(|e| format!("Stage source input: {e}"))?;
        }
        Ok(())
    }
    pub fn stdin(&self) -> Result<File, String> {
        OpenOptions::new()
            .read(true)
            .custom_flags(0x20000)
            .open(self.root.join("source.code"))
            .map_err(|e| format!("Open staged source stdin: {e}"))
    }
    pub fn clean(&mut self) -> Result<(), String> {
        fs::remove_dir_all(&self.root).map_err(|e| format!("Clean owned source workspace: {e}"))?;
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
