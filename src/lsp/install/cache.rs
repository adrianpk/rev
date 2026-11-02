use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) struct Cache {
    pub directory: PathBuf,
    pub root: PathBuf,
    identity: (u64, u64),
    removed: bool,
}

impl Cache {
    pub fn new(root: &Path) -> Result<Self, String> {
        fs::create_dir_all(root).map_err(|e| format!("create cache root: {e}"))?;
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let cache = root.join("lsp");
        super::storage::contained(&root, &cache, false)?;
        fs::create_dir_all(&cache).map_err(|e| format!("create LSP cache: {e}"))?;
        super::tree::bytes(&cache, super::CACHE_BYTES)?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let directory = cache.join(format!("job-{:x}-{nonce:x}", std::process::id()));
        fs::create_dir(&directory).map_err(|e| format!("create unique owned cache: {e}"))?;
        let metadata = fs::symlink_metadata(&directory).map_err(|e| e.to_string())?;
        Ok(Self {
            directory,
            root: cache,
            identity: (metadata.dev(), metadata.ino()),
            removed: false,
        })
    }

    pub fn cleanup(&mut self) -> Result<(), String> {
        let metadata = fs::symlink_metadata(&self.directory)
            .map_err(|e| format!("inspect owned cache cleanup: {e}"))?;
        if !metadata.is_dir() || (metadata.dev(), metadata.ino()) != self.identity {
            return Err("owned cache identity changed; preserve it for inspection".into());
        }
        fs::remove_dir_all(&self.directory).map_err(|e| format!("remove owned cache: {e}"))?;
        self.removed = true;
        Ok(())
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        if !self.removed {
            let _ = self.cleanup();
        }
    }
}
