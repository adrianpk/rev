//! Recipe-owned runtime paths survive user-option inheritance. The registry
//! admits those paths before a ready installation can be selected again.
use super::{Installation, Launch, State};
use crate::config::Server;

pub(crate) fn apply(
    installation: &Installation,
    launch: &Launch,
    server: &mut Server,
) -> Result<(), String> {
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
    if installation.recipe == "ols" {
        server.env.insert(
            "OLS_BUILTIN_FOLDER".into(),
            launch
                .env
                .get("OLS_BUILTIN_FOLDER")
                .ok_or("OLS builtin path missing")?
                .clone(),
        );
        super::ols::snippets(&mut server.initialization_options)?;
    }
    Ok(())
}

pub(super) fn validate(installation: &Installation, data: &std::path::Path) -> Result<(), String> {
    if installation.status != State::Ready {
        return Ok(());
    }
    match installation.recipe.as_str() {
        "ruby-lsp" => ruby(installation, data),
        "ols" => ols(installation, data),
        _ => Ok(()),
    }
}

fn ruby(installation: &Installation, data: &std::path::Path) -> Result<(), String> {
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

fn ols(installation: &Installation, data: &std::path::Path) -> Result<(), String> {
    if !super::ols::revision(&installation.version)
        || installation.evidence != format!("git-revision:{}", installation.version)
    {
        return Err("ready OLS revision differs from its evidence".into());
    }
    let path = data.join(&installation.payload).join("source/builtin");
    super::storage::contained(data, &path, true)?;
    if !path.is_dir() {
        return Err("ready OLS builtin collection missing".into());
    }
    let expected = super::package::path(&path)?;
    for launch in &installation.launches {
        if launch.env.get("OLS_BUILTIN_FOLDER") != Some(&expected)
            || ["enable_snippets", "enable_fake_methods"]
                .iter()
                .any(|key| {
                    launch
                        .initialization_options
                        .get(*key)
                        .and_then(toml::Value::as_bool)
                        != Some(false)
                })
        {
            return Err(
                "ready OLS requires its owned builtin path and disabled snippet-only options"
                    .into(),
            );
        }
    }
    Ok(())
}
