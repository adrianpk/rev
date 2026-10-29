mod decisions;

pub use decisions::{Confirmation, SaveRequest};

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use ropey::Rope;

use crate::file::{self, Location, Outcome, Output, Publication};

use super::{Buffer, BufferId, Buffers, Stamp, State};

/// Stable captured bytes and logical state. Later buffer edits cannot alter
/// this private COW snapshot or be acknowledged by its successful publication.
pub struct Save {
    text: Rope,
    bom: bool,
    stamp: Stamp,
    state: State,
    prior: Option<Location>,
    pub(crate) target: Location,
}

impl Save {
    fn capture(buffer: &Buffer, target: Location) -> Self {
        Self {
            text: buffer.text.clone(),
            bom: buffer.bom,
            stamp: buffer.stamp(),
            state: buffer.state,
            prior: buffer.file.clone(),
            target,
        }
    }

    pub fn stamp(&self) -> Stamp {
        self.stamp
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn target(&self) -> &Path {
        self.target.path()
    }

    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        file::write_text(self.text.slice(..), self.bom, writer)
    }
}

impl Buffers {
    /// Capture an associated destination, or an explicit unoccupied new target.
    /// Occupied alternate targets require a separate overwrite decision.
    pub fn prepare_save(&self, id: BufferId, target: Option<&Path>) -> io::Result<Save> {
        let buffer = self.get(id).map_err(io::Error::other)?;
        writable(buffer)?;
        let target = destination(buffer, target)?;
        self.target_available(id, target.path())?;
        Ok(Save::capture(buffer, target))
    }

    pub fn save(&mut self, id: BufferId) -> io::Result<Outcome> {
        let save = self.prepare_save(id, None)?;
        Ok(self.publish_save(save))
    }

    pub fn publish_save(&mut self, save: Save) -> Outcome {
        self.publish_with(save, &mut file::Files)
    }

    /// The default output is the real filesystem; boundary overrides support
    /// failure evidence without adding runtime fault switches or fake commands.
    pub fn publish_with(&mut self, save: Save, output: &mut impl Output) -> Outcome {
        let label = match self.admit_save(&save) {
            Ok(label) => label,
            Err(error) => return Outcome::failed(save.target(), error),
        };
        let buffer = match self.entries.get_mut(&save.stamp.buffer) {
            Some(buffer) => buffer,
            None => {
                return Outcome::failed(
                    save.target(),
                    io::Error::other("save buffer is no longer live"),
                );
            }
        };
        let outcome = file::publish(&save, output);
        if outcome.publication() != Publication::NotPublished {
            reconcile(buffer, &mut self.files, &save, &outcome, label);
            buffer.apply_settings(&self.settings);
        }
        outcome
    }

    fn admit_save(&self, save: &Save) -> io::Result<String> {
        let buffer = self.get(save.stamp.buffer).map_err(io::Error::other)?;
        writable(buffer)?;
        if buffer.file != save.prior {
            return Err(io::Error::other(
                "buffer association changed after save capture",
            ));
        }
        self.target_available(buffer.id(), save.target())?;
        let base = save.target.basename().to_string_lossy();
        if buffer.name() == base
            || buffer
                .file
                .as_ref()
                .is_some_and(|file| file.path() == save.target())
        {
            return Ok(buffer.name.clone());
        }
        Ok(self.label(&base))
    }

    fn target_available(&self, id: BufferId, path: &Path) -> io::Result<()> {
        if self.files.get(path).is_some_and(|owner| *owner != id) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "destination is visited by another live buffer",
            ));
        }
        Ok(())
    }
}

fn writable(buffer: &Buffer) -> io::Result<()> {
    if buffer.is_transient() {
        return Err(io::Error::other(
            "Transient capture draft cannot be saved as a source file",
        ));
    }
    if buffer.is_readonly() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "buffer is read-only",
        ));
    }
    Ok(())
}

fn destination(buffer: &Buffer, target: Option<&Path>) -> io::Result<Location> {
    let Some(path) = target else {
        return buffer.file.clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer has no associated file destination",
            )
        });
    };
    let target = file::resolve(path)?;
    if let Some(prior) = &buffer.file
        && prior.path() == target.path()
    {
        return Ok(prior.clone());
    }
    if target.is_present() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "occupied alternate destination requires confirmation",
        ));
    }
    Ok(target)
}

fn reconcile(
    buffer: &mut Buffer,
    index: &mut BTreeMap<PathBuf, BufferId>,
    save: &Save,
    outcome: &Outcome,
    label: String,
) {
    if let Some(prior) = &buffer.file {
        index.remove(prior.path());
    }
    index.insert(save.target().into(), buffer.id());
    buffer.file = Some(save.target.published(outcome.observed.clone()));
    buffer.name = label;
    buffer.break_typing();
    if outcome.is_complete() {
        buffer.saved = save.state;
        buffer.warning = None;
    } else {
        let error = outcome
            .error()
            .map(ToString::to_string)
            .unwrap_or_else(|| "completion unresolved".into());
        buffer.warning = Some(format!("published {:?}: {error}", outcome.target()));
    }
}
