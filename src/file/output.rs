use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use crate::buffer::Save;

use super::{Observation, observe};

/// Real filesystem boundaries for saving. Overrides may inject I/O failures;
/// an error from replace must mean that replacement did not occur. A later
/// failure belongs to sync_directory or observe, after publication is retained.
pub trait Output {
    fn write(&mut self, save: &Save, file: &mut File) -> io::Result<()> {
        save.write(file)
    }

    fn flush(&mut self, file: &mut File) -> io::Result<()> {
        file.flush()
    }

    fn sync_file(&mut self, file: &File) -> io::Result<()> {
        file.sync_all()
    }

    fn replace(&mut self, temporary: &Path, target: &Path) -> io::Result<()> {
        fs::rename(temporary, target)
    }

    fn sync_directory(&mut self, directory: &File) -> io::Result<()> {
        directory.sync_all()
    }

    fn observe(&mut self, path: &Path) -> io::Result<Observation> {
        observe(path)?.ok_or_else(|| io::Error::other("published destination is missing"))
    }

    fn remove(&mut self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}

pub struct Files;

impl Output for Files {}
