use std::io;
use std::path::{Path, PathBuf};

use crate::file;

use super::{Buffer, BufferId, Buffers, Error};

pub(crate) enum Visit {
    Existing(BufferId),
    New(Box<Buffer>, PathBuf),
}

impl Visit {
    pub(crate) fn buffer_mut<'a>(
        &'a mut self,
        buffers: &'a mut Buffers,
    ) -> Result<&'a mut Buffer, Error> {
        match self {
            Self::Existing(id) => buffers.get_mut(*id),
            Self::New(buffer, _) => Ok(buffer),
        }
    }

    pub(crate) fn buffer<'a>(&'a self, buffers: &'a Buffers) -> Result<&'a Buffer, Error> {
        match self {
            Self::Existing(id) => buffers.get(*id),
            Self::New(buffer, _) => Ok(buffer),
        }
    }
}

impl Buffers {
    /// Reuse one live canonical destination or install validated ingress. An
    /// absent destination starts empty and clean without creating disk entries.
    pub fn visit(&mut self, path: &Path) -> io::Result<BufferId> {
        let visit = self.prepare_visit(path)?;
        Ok(self.publish_visit(visit))
    }

    pub(crate) fn prepare_visit(&self, path: &Path) -> io::Result<Visit> {
        let location = file::resolve(path)?;
        if let Some(id) = self.files.get(location.path()) {
            self.get(*id).map_err(io::Error::other)?;
            return Ok(Visit::Existing(*id));
        }
        let mut buffer = visiting(&location, &self.settings.settings().files)?;
        let path = location.path().to_path_buf();
        buffer.name = location.basename().to_string_lossy().into_owned();
        buffer.file = Some(location);
        buffer.apply_settings(&self.settings);
        Ok(Visit::New(Box::new(buffer), path))
    }

    pub(crate) fn prepare_observed_visit(
        &self,
        path: &Path,
        loaded: file::Loaded,
        expected: &file::Observation,
    ) -> io::Result<Visit> {
        let location = file::resolve(path)?;
        if location.observation() != Some(expected) {
            return Err(io::Error::other(
                "Agenda disk source changed before visiting; refresh required",
            ));
        }
        if let Some(id) = self.files.get(location.path()) {
            let buffer = self.get(*id).map_err(io::Error::other)?;
            let span = buffer
                .span(0..buffer.len_chars())
                .map_err(io::Error::other)?;
            if buffer.slice(&span).map_err(io::Error::other)? != loaded.observed_text() {
                return Err(io::Error::other(
                    "Open source differs from the agenda; refresh required",
                ));
            }
            return Ok(Visit::Existing(*id));
        }
        let mut buffer = Buffer::from_loaded(loaded).map_err(io::Error::other)?;
        let path = location.path().to_owned();
        buffer.name = location.basename().to_string_lossy().into_owned();
        buffer.file = Some(location);
        buffer.apply_settings(&self.settings);
        Ok(Visit::New(Box::new(buffer), path))
    }
    pub(crate) fn prepare_missing_visit(&self, path: &Path) -> io::Result<Visit> {
        let location = file::resolve(path)?;
        if location.is_present() || self.files.contains_key(location.path()) {
            return Err(io::Error::other(
                "Missing destination changed before publication",
            ));
        }
        let mut buffer = visiting(&location, &self.settings.settings().files)?;
        let path = location.path().to_owned();
        buffer.name = location.basename().to_string_lossy().into_owned();
        buffer.file = Some(location);
        buffer.apply_settings(&self.settings);
        Ok(Visit::New(Box::new(buffer), path))
    }

    pub(crate) fn publish_visit(&mut self, visit: Visit) -> BufferId {
        match visit {
            Visit::Existing(id) => id,
            Visit::New(buffer, path) => {
                self.files.insert(path, buffer.id());
                self.insert(*buffer)
            }
        }
    }

    pub(super) fn label(&self, base: &str) -> String {
        let mut label = base.to_owned();
        // At most N live labels can occupy these N+1 distinct candidates.
        for number in 1..=self.entries.len() {
            if !self.entries.values().any(|buffer| buffer.name() == label) {
                return label;
            }
            label = format!("{base}<{}>", number + 1);
        }
        label
    }
}

fn visiting(location: &file::Location, files: &crate::config::Files) -> io::Result<Buffer> {
    if location.is_present() {
        return Buffer::from_loaded(file::load(location.path())?).map_err(io::Error::other);
    }
    Buffer::new("", files).map_err(io::Error::other)
}
