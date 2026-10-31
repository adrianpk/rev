use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use super::{Observation, observe};

/// Canonical existing referent or normalized intended destination. Presence
/// records resolution time; it is not a claim that disk cannot change later.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    path: PathBuf,
    basename: OsString,
    present: bool,
    observed: Option<Observation>,
}

impl Location {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The lossless name used on the first visit, before canonicalizing aliases.
    /// It is independent of both destination identity and unique display labels.
    pub fn basename(&self) -> &OsStr {
        &self.basename
    }

    pub fn is_present(&self) -> bool {
        self.present
    }

    pub fn observation(&self) -> Option<&Observation> {
        self.observed.as_ref()
    }

    pub(crate) fn missing(&self) -> Self {
        Self {
            path: self.path.clone(),
            basename: self.basename.clone(),
            present: false,
            observed: None,
        }
    }

    pub(crate) fn relocated(&self, path: PathBuf) -> Self {
        let basename = if path.file_name() == self.path.file_name() {
            self.basename.clone()
        } else {
            path.file_name().unwrap_or(path.as_os_str()).to_owned()
        };
        Self {
            path,
            basename,
            ..self.clone()
        }
    }

    /// Refresh disk evidence for an existing destination without replacing the
    /// first visiting alias. An alternate destination keeps its requested name.
    pub(crate) fn refreshed(&self, observed: Self) -> Self {
        if self.path == observed.path {
            Self {
                basename: self.basename.clone(),
                ..observed
            }
        } else {
            observed
        }
    }

    pub(crate) fn published(&self, observed: Option<Observation>) -> Self {
        Self {
            path: self.path.clone(),
            basename: self.basename.clone(),
            present: true,
            observed,
        }
    }
}

/// Resolve existing prefixes before processing `..`, so symlink parents keep
/// filesystem semantics. Missing suffixes are normalized without creating them.
pub fn resolve(path: &Path) -> io::Result<Location> {
    resolve_path(path)
        .map_err(|error| io::Error::new(error.kind(), format!("visit {path:?}: {error}")))
}

fn resolve_path(path: &Path) -> io::Result<Location> {
    let resolved = components(path)?;
    let observed = observe(&resolved)?;
    let present = observed.is_some();
    Ok(Location {
        path: resolved,
        basename: path.file_name().unwrap_or(path.as_os_str()).to_owned(),
        present,
        observed,
    })
}

/// Directory opening shares component/alias resolution with file visits while
/// retaining the regular-file invariant of Location and its disk observations.
pub(crate) fn directory(path: &Path) -> io::Result<PathBuf> {
    let resolved = components(path)?;
    if !fs::symlink_metadata(&resolved)?.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            "expected a directory",
        ));
    }
    Ok(resolved)
}

fn components(path: &Path) -> io::Result<PathBuf> {
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
    Ok(resolved)
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
