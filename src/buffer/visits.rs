use std::io;
use std::path::{Path, PathBuf};

use crate::file;

use super::{Buffer, BufferId, Buffers, Error};

pub(crate) enum Visit {
    Existing(BufferId),
    New(Box<Buffer>, PathBuf),
}

impl Visit {
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
        let mut buffer = visiting(&location)?;
        let path = location.path().to_path_buf();
        buffer.name = location.basename().to_string_lossy().into_owned();
        buffer.file = Some(location);
        buffer.select_syntax();
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

fn visiting(location: &file::Location) -> io::Result<Buffer> {
    if location.is_present() {
        return Buffer::from_loaded(file::load(location.path())?).map_err(io::Error::other);
    }
    Buffer::new("").map_err(io::Error::other)
}
