use std::fmt::Write;
use std::path::Path;

use crate::config::{Preference, Roots, Server, SettingsEdit};
use crate::lsp::install::{self, Installation, Outcome, Registry, State, Storage};
use crate::session::{Action, Session};

pub(in crate::session) enum Choice {
    Install(String),
    Ready(String),
}

pub(super) fn choices(language: &str) -> Vec<String> {
    let Ok(roots) = Roots::from_env() else {
        return Vec::new();
    };
    let Ok(registry) = Registry::load(&roots.data) else {
        return Vec::new();
    };
    registry
        .installations
        .iter()
        .filter(|i| compatible(i, language))
        .map(|i| format!("ready:{}", i.id))
        .collect()
}

fn compatible(installation: &Installation, language: &str) -> bool {
    installation.status == State::Ready
        && (language.is_empty()
            || installation
                .launches
                .iter()
                .any(|p| p.languages.iter().any(|l| l == language)))
}

pub(super) fn recipes(language: &str) -> Vec<String> {
    crate::lsp::catalog::RECIPES
        .iter()
        .filter(|r| install::SUPPORTED_RECIPES.contains(&r.id))
        .filter(|r| language.is_empty() || r.launch(language).is_some())
        .map(|r| r.id.to_owned())
        .collect()
}

pub(super) fn languages(id: &str) -> Result<Vec<String>, String> {
    let mut roots = Roots::from_env().map_err(|e| e.to_string())?;
    roots.data = roots.data.canonicalize().unwrap_or(roots.data);
    let registry = Registry::load(&roots.data)?;
    let installation = registry
        .installations
        .iter()
        .find(|i| i.id == id && i.status == State::Ready)
        .ok_or("ready installation unavailable; inspect lsp-servers")?;
    Ok(installation
        .launches
        .iter()
        .flat_map(|p| p.languages.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect())
}

impl Session {
    pub(in crate::session) fn launch_installation(
        &mut self,
        choice: Choice,
        edit: SettingsEdit,
        language: String,
        inherited: std::collections::BTreeMap<String, Server>,
    ) -> Action {
        let roots = match Roots::from_env() {
            Ok(roots) => roots,
            Err(e) => return self.error(&e.to_string()),
        };
        let source = match self.buffer() {
            Ok(buffer) => buffer.stamp(),
            Err(e) => return self.error(&e.to_string()),
        };
        let result = self
            .installation
            .start(source, roots, move |mut storage, cancel| {
                let mut outcome = match choice {
                    Choice::Install(recipe) => {
                        install::install(&mut storage, &recipe, &language, inherited, cancel)?
                    }
                    Choice::Ready(id) => ready(&storage, &id)?,
                };
                if cancel.is_cancelled() {
                    outcome
                        .message
                        .push_str("; ready inventory retained, not selected after cancellation");
                    return Ok(outcome);
                }
                select(&storage, &mut outcome, edit, &language);
                Ok(outcome)
            });
        match result {
            Ok(()) => {
                self.installation_notice =
                    "Checking requested server; installation work is active".into();
                self.message("Server management started; editing remains available, lsp-mode controls analysis")
            }
            Err(error) => self.error(&error),
        }
    }

    pub(super) fn poll_installation(&mut self) -> Action {
        let Some(result) = self.installation.poll() else {
            return Action::Wait;
        };
        match result {
            Ok(outcome) => {
                if let Some(publication) = outcome.selection {
                    self.apply_lsp_settings(&publication.snapshot);
                }
                self.installation_notice = outcome.message;
                self.message(&self.installation_notice.clone())
            }
            Err(error) => {
                self.installation_notice = format!("Server management failed: {error}");
                self.error(&self.installation_notice.clone())
            }
        }
    }

    pub(super) fn server_inventory(&self, path: Option<&std::ffi::OsStr>) -> String {
        let result = Roots::from_env()
            .map_err(|e| e.to_string())
            .and_then(|mut roots| {
                roots.data = roots.data.canonicalize().unwrap_or(roots.data);
                Registry::load(&roots.data).map(|r| (roots, r))
            });
        let snapshot = self.buffers.snapshot();
        let mut text = match &result {
            Ok((roots, registry)) => crate::lsp::catalog::with_installations(
                snapshot,
                path,
                (std::env::consts::OS, std::env::consts::ARCH),
                (registry, &roots.data),
            ),
            Err(_) => crate::lsp::catalog::inventory(
                snapshot,
                path,
                std::env::consts::OS,
                std::env::consts::ARCH,
            ),
        };
        let _ = writeln!(
            text,
            "\nManagement job: {}\n{}",
            self.installation.phase().label(),
            self.installation_notice
        );
        match result {
            Ok((roots, registry)) => inventory(&mut text, &roots.data, &registry, snapshot),
            Err(error) => {
                let _ = writeln!(
                    text,
                    "Managed registry unavailable: {error}. Existing configured servers remain usable."
                );
            }
        }
        text
    }
}

fn ready(storage: &Storage, id: &str) -> Result<Outcome, String> {
    let installation = storage
        .registry()
        .installations
        .iter()
        .find(|i| i.id == id && i.status == State::Ready)
        .ok_or("ready installation unavailable; inspect lsp-servers")?
        .clone();
    Ok(Outcome {
        message: format!(
            "Managed {} {} ready",
            installation.recipe, installation.version
        ),
        installation,
        selection: None,
    })
}

fn select(storage: &Storage, outcome: &mut Outcome, mut edit: SettingsEdit, language: &str) {
    let result = selection(storage, &outcome.installation, &mut edit, language)
        .and_then(|()| storage.select(&edit));
    match result {
        Ok(publication) => {
            outcome
                .message
                .push_str("; selected and saved; lsp-mode controls activation");
            if let Some(warning) = &publication.warning {
                outcome.message.push_str(&format!("; {warning}"));
            }
            outcome.selection = Some(publication);
        }
        Err(error) => outcome.message.push_str(&format!(
            "; ready inventory retained, not selected: {error}"
        )),
    }
}

fn selection(
    storage: &Storage,
    installation: &Installation,
    edit: &mut SettingsEdit,
    language: &str,
) -> Result<(), String> {
    let launch = installation
        .launches
        .iter()
        .find(|p| p.languages.iter().any(|l| l == language))
        .ok_or("ready installation has no compatible launch profile")?;
    let mut server = launch.server(&storage.roots().data.join(&installation.payload));
    let snapshot = edit.snapshot().map_err(|e| e.to_string())?;
    if let Some((_, current)) = snapshot.settings().lsp.selected(language) {
        // Reusing one shared package for another language retains that language's
        // current choices rather than replaying the initial acquisition's values.
        server.env.clone_from(&current.env);
        server.root_markers.clone_from(&current.root_markers);
        server
            .initialization_options
            .clone_from(&current.initialization_options);
        server.settings.clone_from(&current.settings);
    }
    let name = profile_name(&snapshot, &server, (&launch.id, &installation.id));
    edit.register(&name, &server);
    edit.prefer(language, &Preference::Server(name));
    Ok(())
}

fn profile_name(
    snapshot: &crate::config::Snapshot,
    server: &Server,
    identity: (&str, &str),
) -> String {
    if let Some((name, _)) = snapshot
        .settings()
        .lsp
        .servers
        .iter()
        .find(|(_, existing)| *existing == server)
    {
        return name.clone();
    }
    let (profile, installation) = identity;
    if !snapshot.settings().lsp.servers.contains_key(profile) {
        return profile.into();
    }
    // Never overwrite a custom profile whose name matches a managed launch ID.
    (0..=32)
        .map(|suffix| format!("{profile}-{installation}-{suffix}"))
        .find(|name| !snapshot.settings().lsp.servers.contains_key(name))
        .expect("at most 32 admitted existing profiles")
}

fn inventory(
    text: &mut String,
    data: &Path,
    registry: &Registry,
    snapshot: &crate::config::Snapshot,
) {
    text.push_str("\nManaged installations\nRecipe | Version | Readiness | Selected configured profiles | ID\n");
    let data = data.canonicalize().unwrap_or_else(|_| data.to_owned());
    for installation in &registry.installations {
        let selected = selected_profiles(installation, &data, snapshot);
        let state = match installation.status {
            State::Ready => "ready (verified transport)",
            State::Preparing => "incomplete (preparing; never resumed)",
            State::Incomplete => "incomplete; not selectable",
        };
        let _ = writeln!(
            text,
            "{} | {} | {state} | {} | {}",
            cell(&installation.recipe),
            cell(&installation.version),
            if selected.is_empty() {
                "not selected"
            } else {
                &selected
            },
            cell(&installation.id)
        );
    }
    text.push_str("Ready payloads remain independent of active analysis. Only an explicit choice selects them.\n");
}

fn selected_profiles(
    installation: &Installation,
    data: &Path,
    snapshot: &crate::config::Snapshot,
) -> String {
    installation
        .launches
        .iter()
        .flat_map(|launch| {
            let command = data
                .join(&installation.payload)
                .join(&launch.executable)
                .to_string_lossy()
                .into_owned();
            snapshot
                .settings()
                .lsp
                .servers
                .iter()
                .filter(move |(_, s)| s.command == command)
                .filter(|(name, s)| {
                    s.languages.iter().any(|l| {
                        snapshot
                            .settings()
                            .lsp
                            .selected(l)
                            .is_some_and(|(selected, _)| selected == *name)
                    })
                })
                .map(|(name, _)| cell(name))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn cell(value: &str) -> String {
    value
        .chars()
        .take(128)
        .flat_map(char::escape_default)
        .collect()
}
