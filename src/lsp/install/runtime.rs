//! Recipe-owned runtime paths survive user-option inheritance. The registry
//! admits those paths before a ready installation can be selected again.
use super::{Installation, Launch, State};
use crate::config::Server;

pub(crate) fn apply(installation: &Installation, launch: &Launch, server: &mut Server) {
    if installation.recipe == "ruby-lsp" {
        for key in [
            "GEM_HOME",
            "GEM_PATH",
            "BUNDLE_GEMFILE",
            "BUNDLE_FROZEN",
            "BUNDLE_IGNORE_CONFIG",
            "BUNDLE_DISABLE_VERSION_CHECK",
            "BUNDLE_USER_HOME",
            "RUBYGEMS_GEMDEPS",
        ] {
            if let Some(value) = launch.env.get(key) {
                server.env.insert(key.into(), value.clone());
            }
        }
    }
}

pub(super) fn validate(installation: &Installation, data: &std::path::Path) -> Result<(), String> {
    if installation.status != State::Ready || installation.recipe != "ruby-lsp" {
        return Ok(());
    }
    let expected = super::ruby::runtime_env(&data.join(&installation.payload))?;
    for launch in &installation.launches {
        if expected
            .iter()
            .any(|(key, value)| launch.env.get(key) != Some(value))
        {
            return Err(
                "ready Ruby installation lacks its isolated runtime paths and frozen bundle".into(),
            );
        }
    }
    Ok(())
}
