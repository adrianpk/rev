use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::config::Snapshot;
use crate::file::{self, Location, Observation, Rename, Renamed};

use super::{Buffer, BufferId, Buffers};

struct Relocation {
    prior: Location,
    target: PathBuf,
    before: Option<Observation>,
    label: String,
}

impl Buffers {
    /// All path/owner checks precede disk mutation. Only a successful rename
    /// publishes associations; text, revisions, views and saved states survive.
    pub fn rename(&mut self, rename: Rename, name: &str) -> io::Result<Renamed> {
        let target = rename.destination(name)?;
        self.rename_target(&rename, &target)?;
        let changes = self.rename_changes(&rename, &target)?;
        rename.publish(&target)?;
        let warning = self.relocate(changes);
        Ok(Renamed {
            source: rename.source().into(),
            target,
            warning,
        })
    }

    fn rename_target(&self, rename: &Rename, target: &Path) -> io::Result<()> {
        let occupied = self
            .files
            .keys()
            .any(|path| path == target || (rename.is_directory() && path.starts_with(target)));
        if occupied {
            return Err(file::rename::failure(
                target,
                io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "destination is visited by another live buffer",
                ),
            ));
        }
        Ok(())
    }

    fn rename_changes(
        &self,
        rename: &Rename,
        target: &Path,
    ) -> io::Result<BTreeMap<BufferId, Relocation>> {
        let mut changes = BTreeMap::new();
        if rename.is_symlink() {
            return Ok(changes);
        }
        for buffer in self.entries.values() {
            if let Some(change) = self.rename_change(buffer, rename, target)? {
                changes.insert(buffer.id(), change);
            }
        }
        Ok(changes)
    }

    fn rename_change(
        &self,
        buffer: &Buffer,
        rename: &Rename,
        target: &Path,
    ) -> io::Result<Option<Relocation>> {
        let Some(prior) = &buffer.file else {
            return Ok(None);
        };
        let Ok(suffix) = prior.path().strip_prefix(rename.source()) else {
            return Ok(None);
        };
        if !rename.is_directory() && !suffix.as_os_str().is_empty() {
            return Ok(None);
        }
        let target = if suffix.as_os_str().is_empty() {
            target.to_path_buf()
        } else {
            target.join(suffix)
        };
        let before =
            file::observe(prior.path()).map_err(|e| file::rename::failure(prior.path(), e))?;
        let base = target
            .file_name()
            .unwrap_or(target.as_os_str())
            .to_string_lossy();
        let label = if rename.is_directory() || buffer.name() == base {
            buffer.name.clone()
        } else {
            self.label(&base)
        };
        Ok(Some(Relocation {
            prior: prior.clone(),
            target,
            before,
            label,
        }))
    }

    fn relocate(&mut self, mut changes: BTreeMap<BufferId, Relocation>) -> Option<String> {
        let mut first_warning = None;
        for buffer in self.entries.values_mut() {
            if let Some(change) = changes.remove(&buffer.id()) {
                let warning = apply(buffer, change, &self.settings);
                if first_warning.is_none() {
                    first_warning = warning;
                }
            }
        }
        self.files = self
            .entries
            .values()
            .filter_map(|buffer| {
                buffer
                    .file
                    .as_ref()
                    .map(|file| (file.path().to_path_buf(), buffer.id()))
            })
            .collect();
        first_warning
    }
}

fn apply(buffer: &mut Buffer, change: Relocation, settings: &Snapshot) -> Option<String> {
    let observed = file::observe(&change.target);
    let mut location = change.prior.relocated(change.target.clone());
    if let Ok(Some(after)) = &observed
        && unchanged(&change, after)
    {
        location = location.published(Some(after.clone()));
    }
    buffer.file = Some(location);
    buffer.apply_settings(settings);
    buffer.name = change.label;
    buffer.break_typing();
    let warning = observed
        .err()
        .map(|error| format!("observe renamed {}: {error}", change.target.display()));
    update_warning(buffer, &change.target, warning.as_deref());
    warning
}

fn unchanged(change: &Relocation, after: &Observation) -> bool {
    change.prior.is_present()
        && change.prior.observation() == change.before.as_ref()
        && change
            .before
            .as_ref()
            .is_some_and(|before| before.same_content(after))
}

fn update_warning(buffer: &mut Buffer, path: &Path, observation: Option<&str>) {
    if let Some(prior) = &buffer.warning {
        buffer.warning = Some(format!("{prior}; renamed to {}", path.display()));
    }
    if let Some(warning) = observation {
        let prior = buffer.warning.take().unwrap_or_default();
        buffer.warning = Some(format!("{prior} {warning}").trim().into());
    }
}
