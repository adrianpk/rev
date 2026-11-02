//! Shared package publication and empty-root probing, owned by the installation
//! transaction. These helpers never inspect a user workspace or apply settings.
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Server;
use crate::org::Cancellation;

use super::{Candidate, Launch, Outcome, State, Storage};

pub(super) fn platform() -> Result<(), String> {
    if std::env::consts::OS != "linux" || std::env::consts::ARCH != "x86_64" {
        return Err(
            "unsupported platform: managed package recipes are verified on Linux x86-64".into(),
        );
    }
    Ok(())
}

pub(super) fn path(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| "owned installation path is not UTF-8".into())
}

pub(super) fn text(bytes: &[u8]) -> Result<String, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "runtime identity is not UTF-8")?
        .trim();
    if text.is_empty() || text.len() > 4096 || text.contains('\0') {
        return Err("runtime identity must contain 1..=4096 non-NUL bytes".into());
    }
    Ok(text.into())
}

pub(super) fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = super::helper::owned_file(path)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("write package provenance: {e}"))?;
    fs::File::open(path.parent().ok_or("owned provenance parent missing")?)
        .and_then(|parent| parent.sync_all())
        .map_err(|e| format!("sync package provenance directory: {e}"))
}

pub(super) fn read(payload: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let path = payload.join(relative);
    super::storage::contained(payload, &path, true)?;
    let mut bytes = Vec::new();
    fs::File::open(&path)
        .and_then(|file| {
            file.take(super::REGISTRY_BYTES as u64 + 1)
                .read_to_end(&mut bytes)
        })
        .map_err(|e| format!("read package provenance: {e}"))?;
    if bytes.len() > super::REGISTRY_BYTES {
        return Err("package provenance exceeds 2 MiB".into());
    }
    Ok(bytes)
}

pub(super) fn launch(profile: &crate::lsp::catalog::Launch, inherited: Option<Server>) -> Launch {
    let base = inherited.unwrap_or_else(|| Server {
        languages: Vec::new(),
        command: String::new(),
        args: Vec::new(),
        root_markers: profile.root_markers.iter().map(|s| (*s).into()).collect(),
        initialization_options: toml::Value::Table(Default::default()),
        settings: toml::Value::Table(Default::default()),
        env: Default::default(),
    });
    Launch {
        id: profile.id.into(),
        languages: profile.languages.iter().map(|s| (*s).into()).collect(),
        executable: profile.command.into(),
        args: profile.args.iter().map(|s| (*s).into()).collect(),
        env: base.env,
        root_markers: base.root_markers,
        initialization_options: base.initialization_options,
        settings: base.settings,
    }
}

pub(super) fn probe(candidate: &Candidate, cancel: &Cancellation) -> Result<(), String> {
    candidate.progress.set(super::Phase::Verifying);
    // Admit the complete package tree before executing an extracted entrypoint.
    // In particular, a package-created .bin link cannot escape the payload.
    super::tree::bytes(&candidate.directory, super::PAYLOAD_BYTES)?;
    for (index, launch) in candidate.installation.launches.iter().enumerate() {
        let root = candidate.directory.join(format!("probe-{index}"));
        fs::create_dir(&root).map_err(|e| format!("create empty probe root: {e}"))?;
        let server = launch.server(&candidate.directory);
        super::probe::verify(&server, &root, cancel)
            .map_err(|e| format!("{} protocol probe failed: {e}", launch.id))?;
        super::tree::bytes(&candidate.directory, super::PAYLOAD_BYTES)?;
        fs::remove_dir_all(&root).map_err(|e| format!("clean owned probe root: {e}"))?;
    }
    Ok(())
}

pub(super) fn publish(
    storage: &mut Storage,
    mut candidate: Candidate,
    verified: Result<(), String>,
) -> Result<Outcome, String> {
    if let Err(error) = verified {
        candidate.installation.status = State::Incomplete;
        return Err(match storage.save(candidate.installation) {
            Ok(None) => format!("{error}; candidate retained as incomplete"),
            Ok(Some(warning)) => format!("{error}; candidate marked incomplete; {warning}"),
            Err(save) => {
                format!("{error}; incomplete publication failed: {save}; preserve preparing data")
            }
        });
    }
    candidate.installation.status = State::Ready;
    candidate.installation.verified_at = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs(),
    );
    let warning = storage.save(candidate.installation.clone())?;
    let mut message = format!(
        "{} {} ready",
        candidate.installation.recipe, candidate.installation.version
    );
    if let Some(warning) = warning {
        message.push_str(&format!("; {warning}"));
    }
    Ok(Outcome {
        installation: candidate.installation,
        selection: None,
        message,
    })
}
