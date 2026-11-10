//! Directory-inode leases prevent removal while any rev client owns a launch.
//! No lock file, registry mutation or idle observer is needed for activation.
use std::fs::{self, File};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

pub(crate) fn launch(server: &crate::config::Server, root: &Path) -> Result<Vec<File>, String> {
    let paths = server
        .args
        .iter()
        .chain(server.env.values())
        .map(PathBuf::from)
        .filter(|p| p.is_absolute());
    let payloads: std::collections::BTreeSet<_> = paths
        .chain(executable(server, root))
        .filter_map(|p| p.canonicalize().ok())
        .filter_map(|p| {
            p.ancestors()
                .take(64)
                .find(|p| managed(p))
                .map(Path::to_path_buf)
        })
        .collect();
    payloads
        .into_iter()
        .map(|payload| shared(&payload))
        .collect()
}

fn executable(server: &crate::config::Server, root: &Path) -> Option<PathBuf> {
    let path = Path::new(&server.command);
    if path.is_absolute() {
        return Some(path.to_owned());
    }
    if path.components().count() > 1 {
        return Some(root.join(path));
    }
    let search = server
        .env
        .get("PATH")
        .map(std::ffi::OsString::from)
        .or_else(|| std::env::var_os("PATH"));
    match crate::lsp::catalog::executable(&server.command, search.as_deref()) {
        crate::lsp::catalog::Executable::Available(path) => Some(path),
        _ => None,
    }
}

fn shared(payload: &Path) -> Result<File, String> {
    let file = open(payload)?;
    file.try_lock_shared()
        .map_err(|_| "managed server payload is being removed".to_owned())?;
    same(payload, &file)?;
    Ok(file)
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
