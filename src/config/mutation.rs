//! One OS-owned settings/management lock. A held guard can cover payload and
//! settings publication without recursively acquiring the same lock.
use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub(crate) struct Lock {
    _file: File,
}

impl Lock {
    pub(crate) fn acquire(state: &Path) -> Result<Self, String> {
        let directory = state.join("lsp");
        fs::create_dir_all(&directory).map_err(|e| format!("create LSP lock directory: {e}"))?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(directory.join("mutation.lock"))
            .map_err(|e| format!("open LSP mutation lock: {e}"))?;
        file.try_lock()
            .map_err(|_| "another LSP management operation owns the mutation lock".to_owned())?;
        Ok(Self { _file: file })
    }
}
