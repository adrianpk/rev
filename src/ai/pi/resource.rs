//! A unique private extraction, owned only for the lifetime of one Pi child.
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SOURCE: &str = include_str!("discovery.js");

pub(super) struct Resource {
    pub path: PathBuf,
    directory: PathBuf,
    lease: File,
}

impl Resource {
    pub fn new() -> Result<Self, String> {
        Self::extract().map_err(|_| "Pi internal resource cache is unavailable or unsafe".into())
    }

    fn extract() -> std::io::Result<Self> {
        let parent = parent()?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(std::io::Error::other)?
            .as_nanos();
        let directory = parent.join(format!("pi-{:x}-{nonce:x}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&directory)?;
        let lease = File::open(&directory)?;
        let resource = Self {
            path: directory.join("discovery.js"),
            directory,
            lease,
        };
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&resource.path)?;
        file.write_all(SOURCE.as_bytes())?;
        resource.check()?;
        Ok(resource)
    }

    pub fn check(&self) -> std::io::Result<()> {
        private(&self.directory)?;
        let named = fs::symlink_metadata(&self.directory)?;
        let held = self.lease.metadata()?;
        if named.dev() != held.dev() || named.ino() != held.ino() {
            return Err(std::io::Error::other("resource directory changed"));
        }
        self.check_file(held.uid())
    }

    fn check_file(&self, owner: u32) -> std::io::Result<()> {
        let file = fs::symlink_metadata(&self.path)?;
        if !file.is_file()
            || file.uid() != owner
            || file.mode() & 0o077 != 0
            || file.nlink() != 1
            || file.len() != SOURCE.len() as u64
            || fs::read(&self.path)? != SOURCE.as_bytes()
        {
            return Err(std::io::Error::other("resource changed"));
        }
        Ok(())
    }
}

fn parent() -> std::io::Result<PathBuf> {
    let cache = crate::config::Roots::from_env()?.cache;
    // Missing parents may be created, but never accept symlink namespaces,
    // another owner's directory or a group/world-writable cache ancestor.
    fs::create_dir_all(&cache)?;
    check_ancestors(&cache)?;
    let parent = cache.join("ai");
    match fs::DirBuilder::new().mode(0o700).create(&parent) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(e) => return Err(e),
    }
    private(&parent)?;
    Ok(parent)
}

fn check_ancestors(cache: &Path) -> std::io::Result<()> {
    for parent in cache.ancestors().take_while(|p| p.parent().is_some()) {
        let meta = fs::symlink_metadata(parent)?;
        if !meta.is_dir() || (meta.mode() & 0o022 != 0 && meta.mode() & 0o1000 == 0) {
            return Err(std::io::Error::other("unsafe cache ancestor"));
        }
    }
    Ok(())
}

fn private(path: &Path) -> std::io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_dir()
        || meta.uid() != rustix::process::geteuid().as_raw()
        || meta.mode() & 0o077 != 0
    {
        return Err(std::io::Error::other("unsafe private namespace"));
    }
    Ok(())
}

impl Drop for Resource {
    fn drop(&mut self) {
        if self.check().is_ok() {
            let _ = fs::remove_file(&self.path);
            let _ = fs::remove_dir(&self.directory);
        }
    }
}
