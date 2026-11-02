use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fmt::Write;

use super::{Executable, RECIPES, executable};
use crate::config::Snapshot;

/// Bounded read-only observation of admitted settings and recipe prerequisites.
/// Registry bytes, network endpoints and executable output are never read here.
pub fn inventory(snapshot: &Snapshot, path: Option<&OsStr>, os: &str, arch: &str) -> String {
    render(snapshot, path, (os, arch), None)
}

/// Render already admitted installation facts; this function performs no registry IO.
pub fn with_installations(
    snapshot: &Snapshot,
    path: Option<&OsStr>,
    platform: (&str, &str),
    facts: (&crate::lsp::install::Registry, &std::path::Path),
) -> String {
    render(snapshot, path, platform, Some(facts))
}

fn render(
    snapshot: &Snapshot,
    path: Option<&OsStr>,
    platform: (&str, &str),
    facts: Option<(&crate::lsp::install::Registry, &std::path::Path)>,
) -> String {
    let (os, arch) = platform;
    let mut text = String::from(
        "Language servers\n\nConfigured sources\nLanguage | Selected source | Origin | Version | Executable availability\n",
    );
    let languages: BTreeSet<_> = snapshot.languages().into_iter().collect();
    for language in languages.iter().take(128) {
        if let Some((name, server)) = snapshot.settings().lsp.selected(language) {
            let custom_path = server.env.get("PATH").map(OsStr::new).or(path);
            let status = availability(executable(&server.command, custom_path));
            let (origin, version) = origin(&server.command, facts);
            let _ = writeln!(
                text,
                "{} | {} | {origin} | {} | {status}",
                cell(language),
                cell(name),
                cell(version)
            );
        } else {
            let source = if matches!(
                snapshot.settings().lsp.preferred.get(language),
                Some(crate::config::Preference::Native(false))
            ) {
                "native-only"
            } else {
                "native fallback"
            };
            let _ = writeln!(
                text,
                "{} | {source} | native | - | no server selected",
                cell(language)
            );
        }
    }
    if languages.len() > 128 {
        text.push_str("Additional language rows omitted (128-row limit).\n");
    }
    text.push_str(
        "\nConfigured profiles\nProfile | Languages | Command | Executable availability\n",
    );
    for (name, server) in &snapshot.settings().lsp.servers {
        let languages = server
            .languages
            .iter()
            .take(8)
            .map(|s| cell(s))
            .collect::<Vec<_>>()
            .join(", ");
        let custom_path = server.env.get("PATH").map(OsStr::new).or(path);
        let status = availability(executable(&server.command, custom_path));
        let _ = writeln!(
            text,
            "{} | {languages} | {} | {status}",
            cell(name),
            cell(&server.command)
        );
    }
    text.push_str(
        "\nManaged recipe catalog\nRecipe | Launch profiles | Managed acquisition prerequisites\n",
    );
    for recipe in RECIPES {
        let missing: Vec<_> = recipe
            .prerequisites
            .iter()
            .filter(|p| !matches!(executable(p, path), Executable::Available(_)))
            .copied()
            .collect();
        let readiness = if !recipe.supports(os, arch) {
            "unsupported platform".into()
        } else if missing.is_empty() {
            "executable prerequisites available; runtime features unverified".into()
        } else {
            format!("missing or unresolved prerequisite: {}", missing.join(", "))
        };
        let profiles = recipe
            .launches
            .iter()
            .map(|p| p.id)
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(text, "{} | {profiles} | {readiness}", recipe.id);
    }
    text.push_str("\nManaged payload readiness/version: unverified; installation facts appear separately when available.\nExecutable availability does not prove LSP readiness. lsp-status observes the active file.\nUse lsp-setup for an installed executable; lsp-select-server changes its source.\n");
    text
}

fn cell(value: &str) -> String {
    value
        .chars()
        .take(256)
        .flat_map(char::escape_default)
        .collect()
}

fn availability(value: Executable) -> &'static str {
    match value {
        Executable::Available(_) => "available; protocol unverified",
        Executable::Missing => "missing executable; install it or choose another profile",
        Executable::UnknownRoot => "relative launch/search path; checked at client activation",
        Executable::SearchLimit => "PATH inspection limit reached",
    }
}

fn origin<'a>(
    command: &str,
    facts: Option<(&'a crate::lsp::install::Registry, &std::path::Path)>,
) -> (&'static str, &'a str) {
    let Some((registry, data)) = facts else {
        return ("external/configured", "unknown");
    };
    let installation = registry.installations.iter().find(|installation| {
        installation.launches.iter().any(|launch| {
            data.join(&installation.payload)
                .join(&launch.executable)
                .to_str()
                == Some(command)
        })
    });
    match installation {
        Some(installation) => (
            match installation.status {
                crate::lsp::install::State::Ready => "managed/ready",
                _ => "managed/incomplete",
            },
            &installation.version,
        ),
        None => ("external/configured", "unknown"),
    }
}
