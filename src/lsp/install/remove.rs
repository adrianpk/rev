//! Destructive work requires admitted facts, unchanged settings, no references
//! and an exclusive payload inode lease. Partial deletion remains incomplete.
use super::{Installation, Outcome, State, Storage};
use crate::config::{SettingsEdit, Snapshot};
use crate::org::Cancellation;
use std::fs;
use std::path::Path;

pub fn references(installation: &Installation, snapshot: &Snapshot, data: &Path) -> Vec<String> {
    let payload = data.join(&installation.payload);
    snapshot
        .settings()
        .lsp
        .servers
        .iter()
        .filter(|(_, server)| {
            command_points(server, &payload)
                || server
                    .args
                    .iter()
                    .chain(server.env.values())
                    .any(|s| points(s, &payload))
        })
        .map(|(name, _)| name.clone())
        .collect()
}

fn command_points(server: &crate::config::Server, payload: &Path) -> bool {
    if points(&server.command, payload) {
        return true;
    }
    let search = server
        .env
        .get("PATH")
        .map(std::ffi::OsString::from)
        .or_else(|| std::env::var_os("PATH"));
    match crate::lsp::catalog::executable(&server.command, search.as_deref()) {
        crate::lsp::catalog::Executable::Available(path) => path
            .canonicalize()
            .is_ok_and(|path| path.starts_with(payload)),
        _ => false,
    }
}

fn points(text: &str, payload: &Path) -> bool {
    let path = Path::new(text);
    if !path.is_absolute() {
        return false;
    }
    path.canonicalize()
        .unwrap_or_else(|_| path.to_owned())
        .starts_with(payload)
}

pub fn remove(
    storage: &mut Storage,
    edit: SettingsEdit,
    previous: &Installation,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    preflight(storage, &edit, previous, cancel)?;
    let payload = super::lease::payload(&storage.roots().data, previous);
    let lease = lease(&payload)?;
    let incomplete = mark(storage, previous)?;
    checkpoint(storage, &edit, cancel)?;
    erase(&payload, lease.as_ref())?;
    let warning = storage.forget(&previous.id)?;
    let mut message = format!(
        "Removed installation {}; configured external servers were not changed",
        previous.id
    );
    if let Some(warning) = warning {
        message.push_str(&format!("; {warning}"));
    }
    Ok(Outcome {
        installation: incomplete,
        selection: None,
        message,
    })
}

fn preflight(
    storage: &Storage,
    edit: &SettingsEdit,
    previous: &Installation,
    cancel: &Cancellation,
) -> Result<(), String> {
    super::update::retained(storage, previous)?;
    let snapshot = edit.snapshot().map_err(|e| e.to_string())?;
    let names = references(previous, &snapshot, &storage.roots().data);
    if !names.is_empty() {
        return Err(format!(
            "installation is referenced by profiles: {}; change those profiles before removal",
            names.join(", ")
        ));
    }
    checkpoint(storage, edit, cancel)
}

fn checkpoint(storage: &Storage, edit: &SettingsEdit, cancel: &Cancellation) -> Result<(), String> {
    edit.check_unchanged()?;
    storage.unchanged()?;
    if cancel.is_cancelled() {
        return Err("removal cancelled; any partially removed data remains incomplete".into());
    }
    Ok(())
}

fn mark(storage: &mut Storage, previous: &Installation) -> Result<Installation, String> {
    let mut incomplete = previous.clone();
    incomplete.status = State::Incomplete;
    incomplete.verified_at = None;
    if let Some(warning) = storage.save(incomplete.clone())? {
        return Err(format!("removal not started; {warning}"));
    }
    Ok(incomplete)
}

fn lease(payload: &Path) -> Result<Option<fs::File>, String> {
    match fs::symlink_metadata(payload) {
        Ok(_) => {
            let lease = super::lease::exclusive(payload)?;
            super::tree::bytes(payload, super::PAYLOAD_BYTES)?;
            Ok(Some(lease))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

fn erase(payload: &Path, lease: Option<&fs::File>) -> Result<(), String> {
    if let Some(lease) = lease {
        super::lease::same(payload, lease)?;
        fs::remove_dir_all(payload).map_err(|e| {
            format!("payload removal incomplete; retain record and retry after inspection: {e}")
        })?;
        fs::File::open(payload.parent().ok_or("payload parent missing")?)
            .and_then(|parent| parent.sync_all())
            .map_err(|e|format!("payload removed; registry remains incomplete because directory durability is unconfirmed: {e}"))?;
    }
    Ok(())
}
