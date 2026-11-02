use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) struct Cache {
    pub directory: PathBuf,
    pub root: PathBuf,
    lease: File,
    removed: bool,
}

impl Cache {
    pub fn finish(
        &mut self,
        result: Result<super::Outcome, String>,
    ) -> Result<super::Outcome, String> {
        match (result, self.cleanup()) {
            (Ok(mut outcome), Err(error)) => {
                outcome
                    .message
                    .push_str(&format!("; cache cleanup failed: {error}"));
                Ok(outcome)
            }
            (Err(error), Err(cleanup)) => Err(format!("{error}; cache cleanup failed: {cleanup}")),
            (result, Ok(())) => result,
        }
    }

    pub fn new(root: &Path) -> Result<Self, String> {
        let cache = namespace(root)?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let directory = cache.join(format!("job-{:x}-{nonce:x}", std::process::id()));
        fs::create_dir(&directory).map_err(|e| format!("create unique owned cache: {e}"))?;
        let lease = File::open(&directory).map_err(|e| e.to_string())?;
        lease.try_lock().map_err(|e| e.to_string())?;
        super::package::write(
            &directory.join(super::cache_cleanup::MARKER),
            super::cache_cleanup::OWNER,
        )?;
        Ok(Self {
            directory,
            root: cache,
            lease,
            removed: false,
        })
    }

    pub fn cleanup(&mut self) -> Result<(), String> {
        super::lease::same(&self.directory, &self.lease)?;
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

fn namespace(root: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(root).map_err(|e| format!("create cache root: {e}"))?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let cache = root.join("lsp");
    super::storage::contained(&root, &cache, false)?;
    fs::create_dir_all(&cache).map_err(|e| format!("create LSP cache: {e}"))?;
    let cache = cache.canonicalize().map_err(|e| e.to_string())?;
    super::tree::bytes(&cache, super::CACHE_BYTES)?;
    Ok(cache)
}
