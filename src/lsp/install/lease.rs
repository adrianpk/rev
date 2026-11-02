//! Directory-inode leases prevent removal while any rev client owns a launch.
//! No lock file, registry mutation or idle observer is needed for activation.
use std::fs::{self, File};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

pub(crate) fn launch(server: &crate::config::Server, root: &Path) -> Result<Option<File>, String> {
    let command = &server.command;
    let path = Path::new(command);
    let executable = if path.is_absolute() {
        path.to_owned()
    } else if path.components().count() > 1 {
        root.join(path)
    } else {
        let search = server
            .env
            .get("PATH")
            .map(std::ffi::OsString::from)
            .or_else(|| std::env::var_os("PATH"));
        match crate::lsp::catalog::executable(command, search.as_deref()) {
            crate::lsp::catalog::Executable::Available(path) => path,
            _ => return Ok(None),
        }
    };
    let Ok(executable) = executable.canonicalize() else {
        return Ok(None);
    };
    let Some(payload) = executable.ancestors().take(64).find(|p| managed(p)) else {
        return Ok(None);
    };
    let file = open(payload)?;
    file.try_lock_shared()
        .map_err(|_| "managed server payload is being removed".to_owned())?;
    same(payload, &file)?;
    Ok(Some(file))
}

fn managed(path: &Path) -> bool {
    let Some(recipe) = path.parent() else {
        return false;
    };
    recipe.parent().is_some_and(|p| {
        p.file_name().is_some_and(|s| s == "installations")
            && p.parent()
                .is_some_and(|p| p.file_name().is_some_and(|s| s == "lsp"))
    })
}

pub(super) fn exclusive(payload: &Path) -> Result<File, String> {
    let file = open(payload)?;
    file.try_lock().map_err(|_| {
        "installation is running in a rev client; stop it before removal".to_owned()
    })?;
    same(payload, &file)?;
    Ok(file)
}

fn open(path: &Path) -> Result<File, String> {
    if !fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .is_dir()
        || path.canonicalize().map_err(|e| e.to_string())? != path
    {
        return Err("managed payload directory is aliased; preserve it for inspection".into());
    }
    File::open(path).map_err(|e| format!("open payload lease: {e}"))
}

pub(super) fn same(path: &Path, file: &File) -> Result<(), String> {
    let current = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    let held = file.metadata().map_err(|e| e.to_string())?;
    if !current.is_dir()
        || (current.dev(), current.ino()) != (held.dev(), held.ino())
        || path.canonicalize().map_err(|e| e.to_string())? != path
    {
        return Err("managed payload identity changed; preserve it for inspection".into());
    }
    Ok(())
}

pub(super) fn payload(data: &Path, installation: &super::Installation) -> PathBuf {
    data.join(&installation.payload)
}
