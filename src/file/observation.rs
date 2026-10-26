use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// Conservative disk identity/content/permission observation, not a content hash
/// or a guarantee against changes between the check and replacement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    device: u64,
    inode: u64,
    length: u64,
    modified: (i64, i64),
    changed: (i64, i64),
    pub(crate) mode: u32,
}

impl Observation {
    /// Rename may change ctime; identity, bytes and permissions must still
    /// agree before acknowledging that expected metadata transition.
    pub(crate) fn same_content(&self, other: &Self) -> bool {
        self.device == other.device
            && self.inode == other.inode
            && self.length == other.length
            && self.modified == other.modified
            && self.mode == other.mode
    }
}

/// Observe a regular destination without following a newly substituted symlink.
pub fn observe(path: &Path) -> io::Result<Option<Observation>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => observation(metadata).map(Some),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn observation(metadata: fs::Metadata) -> io::Result<Observation> {
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a regular file",
        ));
    }
    Ok(Observation {
        device: metadata.dev(),
        inode: metadata.ino(),
        length: metadata.len(),
        modified: (metadata.mtime(), metadata.mtime_nsec()),
        changed: (metadata.ctime(), metadata.ctime_nsec()),
        mode: metadata.mode(),
    })
}
