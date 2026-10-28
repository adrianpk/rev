//! Startup-only publication; ordinary location resolution and loading stay pure.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use crate::file::{Files, temporary::Temporary};

use super::{Diagnostic, default_template};

/// Inherit the native sync/cleanup boundaries; configuration never calls the
/// document replacement operation. A link failure cannot replace its target.
trait Output: crate::file::Output {
    fn template(&mut self, file: &mut File) -> io::Result<()> {
        file.write_all(default_template().as_bytes())
    }

    fn link(&mut self, temporary: &Path, root: &Path) -> io::Result<()> {
        fs::hard_link(temporary, root)
    }
}

impl Output for Files {}

pub(super) fn provision(root: &Path) -> Result<(), Diagnostic> {
    publish(root, &mut Files)
        .map_err(|error| Diagnostic::new(root, format!("provision configuration: {error}")))
}

fn publish(root: &Path, output: &mut impl Output) -> io::Result<()> {
    if present(root)? {
        return Ok(());
    }
    let parent = root
        .parent()
        .ok_or_else(|| io::Error::other("configuration has no parent"))?;
    operation("create configuration parents", fs::create_dir_all(parent))?;
    let directory = operation("open configuration directory", File::open(parent))?;
    let mut temporary = operation("create temporary configuration", Temporary::new(parent))?;
    if let Err(error) = install(&mut temporary, root, output) {
        return Err(temporary.failed(error, output));
    }
    operation(
        "synchronize configuration directory",
        output.sync_directory(&directory),
    )
}

fn present(root: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(root) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(io::Error::new(
            error.kind(),
            format!("inspect configuration entry: {error}"),
        )),
    }
}

fn install(temporary: &mut Temporary, root: &Path, output: &mut impl Output) -> io::Result<()> {
    operation(
        "write temporary configuration",
        output.template(&mut temporary.file),
    )?;
    operation(
        "flush temporary configuration",
        output.flush(&mut temporary.file),
    )?;
    operation(
        "synchronize temporary configuration",
        output.sync_file(&temporary.file),
    )?;
    match output.link(&temporary.path, root) {
        Ok(()) => (),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
        Err(error) => return operation("publish absent configuration", Err(error)),
    }
    operation(
        "remove temporary configuration",
        output.remove(&temporary.path),
    )?;
    temporary.disarm();
    Ok(())
}

fn operation<T>(name: &str, result: io::Result<T>) -> io::Result<T> {
    result.map_err(|error| io::Error::new(error.kind(), format!("{name}: {error}")))
}

#[cfg(test)]
#[path = "publication/tests.rs"]
mod tests;
