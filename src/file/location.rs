use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Canonical existing referent or normalized intended destination. Presence
/// records resolution time; it is not a claim that disk cannot change later.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    path: PathBuf,
    present: bool,
}

impl Location {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_present(&self) -> bool {
        self.present
    }
}

/// Resolve existing prefixes before processing `..`, so symlink parents keep
/// filesystem semantics. Missing suffixes are normalized without creating them.
pub fn resolve(path: &Path) -> io::Result<Location> {
    resolve_path(path)
        .map_err(|error| io::Error::new(error.kind(), format!("visit {path:?}: {error}")))
}

fn resolve_path(path: &Path) -> io::Result<Location> {
    if path.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "empty file path",
        ));
    }
    let mut resolved = origin(path)?;
    for component in path.components() {
        match component {
            Component::Normal(name) => append(&mut resolved, name)?,
            Component::ParentDir => parent(&mut resolved)?,
            Component::CurDir => {}
            _ => resolved.push(component.as_os_str()),
        }
    }
    let present = presence(&resolved)?;
    Ok(Location {
        path: resolved,
        present,
    })
}

fn origin(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(PathBuf::new());
    }
    fs::canonicalize(std::env::current_dir()?)
}

fn append(path: &mut PathBuf, name: &OsStr) -> io::Result<()> {
    path.push(name);
    match fs::canonicalize(&path) {
        Ok(canonical) => *path = canonical,
        Err(error) if error.kind() == io::ErrorKind::NotFound => dangling(path)?,
        Err(error) => return Err(error),
    }
    Ok(())
}

fn dangling(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_symlink() => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "symlink referent does not exist",
        )),
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

fn parent(path: &mut PathBuf) -> io::Result<()> {
    match fs::metadata(&path) {
        Ok(metadata) if !metadata.is_dir() => {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                "path component is not a directory",
            ));
        }
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    path.pop();
    Ok(())
}

fn presence(path: &Path) -> io::Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a regular file",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
