use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::buffer::Save;

use super::{Observation, Output, observe, temporary::Temporary};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Publication {
    NotPublished,
    Complete,
    Uncertain,
}

/// Actual publication survives every later completion/observation failure.
#[derive(Debug)]
pub struct Outcome {
    publication: Publication,
    target: PathBuf,
    error: Option<io::Error>,
    pub(crate) observed: Option<Observation>,
}

impl Outcome {
    pub fn publication(&self) -> Publication {
        self.publication
    }
    pub fn is_complete(&self) -> bool {
        self.publication == Publication::Complete
    }
    pub fn target(&self) -> &Path {
        &self.target
    }
    pub fn error(&self) -> Option<&io::Error> {
        self.error.as_ref()
    }

    pub(crate) fn failed(target: &Path, error: io::Error) -> Self {
        Self {
            publication: Publication::NotPublished,
            target: target.into(),
            error: Some(error),
            observed: None,
        }
    }
}

struct Staged {
    temporary: Temporary,
    directory: File,
}

pub(crate) fn publish(save: &Save, output: &mut impl Output) -> Outcome {
    let target = save.target.path();
    let mut staged = match stage(save, output) {
        Ok(staged) => staged,
        Err(error) => return Outcome::failed(target, error),
    };
    let replacement = operation("recheck destination", guard(save)).and_then(|()| {
        operation(
            "replace destination",
            output.replace(&staged.temporary.path, target),
        )
    });
    if let Err(error) = replacement {
        return Outcome::failed(target, staged.temporary.failed(error, output));
    }
    staged.temporary.disarm();
    // No fallible completion step may erase this successful replacement.
    let mut outcome = Outcome {
        publication: Publication::Uncertain,
        target: target.into(),
        error: None,
        observed: None,
    };
    let synced = operation(
        "synchronize destination directory",
        output.sync_directory(&staged.directory),
    );
    let observed = operation("observe published destination", output.observe(target));
    outcome.observed = observed.as_ref().ok().cloned();
    outcome.error = failures(synced, observed.map(|_| ()));
    if outcome.error.is_none() {
        outcome.publication = Publication::Complete;
    }
    outcome
}

fn stage(save: &Save, output: &mut impl Output) -> io::Result<Staged> {
    guard(save)?;
    let parent = save
        .target
        .path()
        .parent()
        .ok_or_else(|| io::Error::other("destination has no parent"))?;
    let directory = File::open(parent)?;
    let mut temporary = Temporary::new(parent)?;
    if let Err(error) = prepare(save, &mut temporary, output) {
        return Err(temporary.failed(error, output));
    }
    Ok(Staged {
        temporary,
        directory,
    })
}

fn guard(save: &Save) -> io::Result<()> {
    let current = observe(save.target.path())?;
    if current.as_ref() != save.target.observation() {
        return Err(io::Error::other(
            "destination changed since last observation",
        ));
    }
    if save.target.is_present() && current.is_none() {
        return Err(io::Error::other(
            "published destination observation is unresolved",
        ));
    }
    if current.is_some() {
        // Check actual write access without truncation, even when replacement
        // would otherwise be possible through a writable parent directory.
        OpenOptions::new().write(true).open(save.target.path())?;
    }
    Ok(())
}

fn prepare(save: &Save, temporary: &mut Temporary, output: &mut impl Output) -> io::Result<()> {
    operation(
        "write temporary output",
        output.write(save, &mut temporary.file),
    )?;
    operation("flush temporary output", output.flush(&mut temporary.file))?;
    let mode = save
        .target
        .observation()
        .map_or(0o600, |observed| observed.mode & 0o777);
    operation(
        "preserve destination permissions",
        temporary
            .file
            .set_permissions(fs::Permissions::from_mode(mode)),
    )?;
    operation(
        "synchronize temporary output",
        output.sync_file(&temporary.file),
    )
}

fn operation<T>(name: &str, result: io::Result<T>) -> io::Result<T> {
    result.map_err(|error| io::Error::new(error.kind(), format!("{name}: {error}")))
}

fn failures(first: io::Result<()>, second: io::Result<()>) -> Option<io::Error> {
    match (first, second) {
        (Ok(()), Ok(())) => None,
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Some(error),
        (Err(first), Err(second)) => {
            Some(io::Error::new(first.kind(), format!("{first}; {second}")))
        }
    }
}
