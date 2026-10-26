use std::io;
use std::path::Path;

use crate::file::{self, Location, Outcome, Output};

use super::{Buffer, BufferId, Buffers, Save, writable};

/// An occupied or externally changed target cannot become publishable without
/// consuming its specific confirmation. Neither variant refreshes consent later.
pub enum SaveRequest {
    Ready(Save),
    Confirm(Confirmation),
}

pub struct Confirmation {
    save: Save,
    message: String,
}

impl Confirmation {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn target(&self) -> &Path {
        self.save.target()
    }
}

impl Buffers {
    /// Resolve a fresh target observation without changing the visited path,
    /// saved state or text. A later filesystem change invalidates this capture.
    pub fn request_save(&self, id: BufferId, target: Option<&Path>) -> io::Result<SaveRequest> {
        let buffer = self.get(id).map_err(io::Error::other)?;
        writable(buffer)?;
        let path = requested_path(buffer, target)?;
        let location = file::resolve(path)?;
        self.target_available(id, location.path())?;
        let reason = reason(buffer, &location, target.is_some());
        let save = Save::capture(buffer, location);
        match reason {
            None => Ok(SaveRequest::Ready(save)),
            Some(reason) => {
                let message = description(reason, path, save.target());
                Ok(SaveRequest::Confirm(Confirmation { save, message }))
            }
        }
    }

    /// Explicit consent consumes only the captured destination/observation.
    /// Normal live-owner, association, permission and disk guards still apply.
    pub fn confirm_save(&mut self, confirmation: Confirmation) -> Outcome {
        self.confirm_save_with(confirmation, &mut file::Files)
    }

    pub fn confirm_save_with(
        &mut self,
        confirmation: Confirmation,
        output: &mut impl Output,
    ) -> Outcome {
        self.publish_save_with(confirmation.save, output)
    }
}

fn requested_path<'a>(buffer: &'a Buffer, target: Option<&'a Path>) -> io::Result<&'a Path> {
    target
        .or_else(|| buffer.file.as_ref().map(Location::path))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer has no associated file destination",
            )
        })
}

fn reason(buffer: &Buffer, target: &Location, explicit: bool) -> Option<&'static str> {
    if let Some(prior) = &buffer.file
        && (prior.path() == target.path() || !explicit)
        && prior != target
    {
        return Some("Destination changed on disk");
    }
    if explicit && target.is_present() {
        return Some("Overwrite occupied destination");
    }
    None
}

fn description(reason: &str, requested: &Path, target: &Path) -> String {
    if requested != target {
        return format!("{reason}? [y/n] target {target:?} (requested {requested:?})");
    }
    format!("{reason}? [y/n] target {target:?}")
}
