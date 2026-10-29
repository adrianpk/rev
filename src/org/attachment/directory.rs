use crate::org::Cancellation;
use std::{
    fs::{self, File, OpenOptions},
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path, PathBuf},
};
// Linux directory descriptors keep operations inside the admitted parent even
// if another process renames it. Every named component rejects symlinks.
const NOFOLLOW: i32 = 0x20000;
const DIRECTORY: i32 = 0x10000;
#[derive(Clone, Debug, PartialEq, Eq)]
struct Identity {
    path: PathBuf,
    device: u64,
    inode: u64,
}
#[derive(Clone)]
pub(super) struct Directory {
    pub path: PathBuf,
    existing: Vec<Identity>,
    missing: Vec<PathBuf>,
}
pub(super) struct Pinned {
    pub handle: File,
    pub path: PathBuf,
}
pub(crate) struct Listing {
    pub names: Vec<String>,
    pub root: PathBuf,
}
impl Directory {
    pub fn observe(path: &Path) -> Result<Self, String> {
        if !path.is_absolute() {
            return Err("Attachment directory must be absolute".into());
        }
        if path.components().count() > 64 {
            return Err("Attachment directory exceeds 64 components".into());
        }
        let mut result = Self {
            path: path.into(),
            existing: Vec::new(),
            missing: Vec::new(),
        };
        let mut current = PathBuf::new();
        for component in path.components() {
            if matches!(component, Component::ParentDir) {
                return Err("Attachment directory traversal rejected".into());
            }
            current.push(component.as_os_str());
            result.component(&current)?;
        }
        if result.existing.len() + result.missing.len() > 64 {
            return Err("Attachment directory exceeds 64 components".into());
        }
        Ok(result)
    }
    fn component(&mut self, path: &Path) -> Result<(), String> {
        match fs::symlink_metadata(path) {
            Ok(m) if m.is_dir() => self.existing.push(Identity {
                path: path.into(),
                device: m.dev(),
                inode: m.ino(),
            }),
            Ok(_) => {
                return Err("Attachment directory component is a symlink or non-directory".into());
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => self.missing.push(path.into()),
            Err(e) => return Err(e.to_string()),
        }
        Ok(())
    }
    pub fn check(&self) -> Result<(), String> {
        let current = Self::observe(&self.path)?;
        if self.existing != current.existing || self.missing != current.missing {
            return Err("Attachment directory changed since selection".into());
        }
        Ok(())
    }
    pub fn materialize(&self, created: &mut Vec<PathBuf>) -> Result<Pinned, String> {
        self.check()?;
        let mut parent = self.anchor()?;
        for path in &self.missing {
            let name = path
                .file_name()
                .ok_or("Missing attachment directory name")?;
            fs::create_dir(parent.entry(name)).map_err(|e| e.to_string())?;
            created.push(path.clone());
            parent.handle.sync_all().map_err(|e| e.to_string())?;
            let next = pin_at(&parent.entry(name), path)?;
            parent = next;
        }
        Ok(parent)
    }
    fn anchor(&self) -> Result<Pinned, String> {
        let expected = self
            .existing
            .last()
            .ok_or("Attachment directory lacks an existing root")?;
        let pinned = pin_at(&expected.path, &expected.path)?;
        let held = pinned.handle.metadata().map_err(|e| e.to_string())?;
        if held.dev() != expected.device || held.ino() != expected.inode {
            return Err("Attachment parent changed before opening".into());
        }
        Ok(pinned)
    }

    pub fn open(&self) -> Result<Pinned, String> {
        self.check()?;
        if !self.missing.is_empty() {
            return Err("Attachment directory does not exist".into());
        }
        self.anchor()
    }
}
impl Pinned {
    pub fn entry(&self, name: &std::ffi::OsStr) -> PathBuf {
        PathBuf::from(format!("/proc/self/fd/{}", self.handle.as_raw_fd())).join(name)
    }
    pub fn current(&self) -> Result<(), String> {
        let current = fs::symlink_metadata(&self.path).map_err(|e| e.to_string())?;
        let held = self.handle.metadata().map_err(|e| e.to_string())?;
        if !current.is_dir() || current.dev() != held.dev() || current.ino() != held.ino() {
            return Err("Pinned attachment directory was replaced".into());
        }
        Ok(())
    }
}
fn pin_at(entry: &Path, path: &Path) -> Result<Pinned, String> {
    let handle = OpenOptions::new()
        .read(true)
        .custom_flags(NOFOLLOW | DIRECTORY)
        .open(entry)
        .map_err(|e| e.to_string())?;
    let pinned = Pinned {
        handle,
        path: path.into(),
    };
    pinned.current()?;
    Ok(pinned)
}
pub(super) fn input(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .read(true)
        .custom_flags(NOFOLLOW)
        .open(path)
        .map_err(|e| e.to_string())
}
impl Listing {
    pub(crate) fn read(context: &super::Context, cancel: &Cancellation) -> Result<Self, String> {
        let root = context.root(None)?;
        let pinned = Directory::observe(&root)?.open()?;
        let mut names = Vec::new();
        let mut bytes = 0;
        for entry in
            fs::read_dir(pinned.entry(std::ffi::OsStr::new("."))).map_err(|e| e.to_string())?
        {
            if cancel.is_cancelled() {
                return Err("Attachment listing cancelled".into());
            }
            let name = listed_name(entry)?;
            bytes += name.len();
            names.push(name);
            listing_limit(names.len(), bytes)?;
        }
        pinned.current()?;
        names.sort();
        Ok(Self { root, names })
    }
}

fn listed_name(entry: std::io::Result<fs::DirEntry>) -> Result<String, String> {
    let entry = entry.map_err(|e| e.to_string())?;
    let name = entry
        .file_name()
        .into_string()
        .map_err(|_| "Attachment filename is not UTF-8")?;
    super::location::name(&name)?;
    if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
        return Err(format!(
            "Attachment {name:?} is a symlink or non-regular entry"
        ));
    }
    Ok(name)
}

fn listing_limit(count: usize, bytes: usize) -> Result<(), String> {
    if count > 4096 || bytes > 1048576 {
        Err("Attachment listing exceeds 4096 files or 1 MiB names".into())
    } else {
        Ok(())
    }
}
