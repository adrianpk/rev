use super::{Listing, directory::Directory};
use crate::file::{self, Observation};
use std::path::{Path, PathBuf};
/// A real listed filename plus its original file and directory observations.
#[derive(Clone)]
pub(crate) struct Selected {
    directory: Directory,
    pub name: String,
    path: PathBuf,
    observation: Observation,
}
impl Listing {
    pub(crate) fn selected(&self, name: &str) -> Result<Selected, String> {
        let index = self
            .names
            .iter()
            .position(|n| n == name)
            .ok_or("Attachment was not in the observed list")?;
        Ok(Selected {
            directory: self.directory.clone(),
            name: name.into(),
            path: self.root.join(name),
            observation: self.observations[index].clone(),
        })
    }
}
impl Selected {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn check(&self) -> Result<(), String> {
        let directory = self.directory.open()?;
        let path = directory.entry(std::ffi::OsStr::new(&self.name));
        if file::observe(&path).map_err(|e| e.to_string())?.as_ref() != Some(&self.observation) {
            return Err("Selected attachment changed since listing".into());
        }
        directory.current()
    }
}
