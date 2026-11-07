use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

use super::{Error, MAX_PATH_BYTES, Root, RootId};

/// Lossless, normalized native path. Missing retained paths remain representable.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NativePath(PathBuf);
impl NativePath {
    pub fn absolute(path: &Path) -> Result<Self, Error> {
        if !path.is_absolute() {
            return Err(Error::Invalid("absolute path"));
        }
        Self::checked(path)
    }
    pub fn relative(path: &Path) -> Result<Self, Error> {
        if path.is_absolute() || path.as_os_str().is_empty() {
            return Err(Error::Invalid("relative path"));
        }
        Self::checked(path)
    }
    fn checked(path: &Path) -> Result<Self, Error> {
        if path.as_os_str().len() > MAX_PATH_BYTES
            || path.as_os_str().as_bytes().contains(&0)
            || path.components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::CurDir | Component::Prefix(_)
                )
            })
        {
            return Err(Error::Invalid("native path"));
        }
        Ok(Self(path.components().collect()))
    }
    pub fn as_path(&self) -> &Path {
        &self.0
    }
    /// New roots must currently be directories; missing stored roots use
    /// `absolute` and never acquire an unrelated cwd fallback.
    pub fn directory(path: &Path) -> Result<Self, Error> {
        let path =
            std::fs::canonicalize(path).map_err(|_| Error::Unavailable("root is unavailable"))?;
        if !path.is_dir() {
            return Err(Error::Invalid("directory root"));
        }
        Self::absolute(&path)
    }
    pub(crate) fn observed(&self) -> PathBuf {
        std::fs::canonicalize(&self.0).unwrap_or_else(|_| self.0.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileReference {
    RootRelative { root: RootId, path: NativePath },
    LooseAbsolute(NativePath),
}
impl FileReference {
    /// Choose the most specific component-wise root, with stored order for a
    /// tie. Resolve aliases before selecting a route; do not read file contents.
    pub fn capture(file: &Path, roots: &[Root]) -> Result<Self, Error> {
        let file =
            std::fs::canonicalize(file).map_err(|_| Error::Unavailable("file is unavailable"))?;
        if !file.is_file() {
            return Err(Error::Invalid("session file"));
        }
        let mut selected = None;
        for root in roots {
            let base = root.path.observed();
            if let Ok(relative) = file.strip_prefix(&base) {
                let depth = base.components().count();
                if selected.as_ref().is_none_or(|(best, _, _)| depth > *best) {
                    selected = Some((depth, root.id.clone(), NativePath::relative(relative)?));
                }
            }
        }
        Ok(match selected {
            Some((_, root, path)) => Self::RootRelative { root, path },
            None => Self::LooseAbsolute(NativePath::absolute(&file)?),
        })
    }
    /// Retained missing files resolve lexically. Existing relative aliases must
    /// still be confined to their owning physical root.
    pub fn resolve(&self, roots: &[Root]) -> Result<PathBuf, Error> {
        match self {
            Self::LooseAbsolute(path) => {
                NativePath::absolute(path.as_path())?;
                Ok(path.observed())
            }
            Self::RootRelative { root, path } => {
                NativePath::relative(path.as_path())?;
                let root = roots
                    .iter()
                    .find(|entry| entry.id == *root)
                    .ok_or(Error::Invalid("session root reference"))?;
                let base = root.path.observed();
                let file = base.join(path.as_path());
                let resolved = match std::fs::canonicalize(&file) {
                    Ok(path) => path,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        crate::file::resolve(&file)
                            .map_err(|_| {
                                Error::Unavailable("cannot resolve retained session path")
                            })?
                            .path()
                            .to_owned()
                    }
                    Err(_) => file,
                };
                if !resolved.starts_with(base) {
                    return Err(Error::Invalid("escaping session path"));
                }
                Ok(resolved)
            }
        }
    }
}
