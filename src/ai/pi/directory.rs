//! Native transmission admission mirrors the resource's filesystem exclusions.
use std::path::{Component, Path, PathBuf};

pub(crate) fn source_path(path: &Path) -> bool {
    path.components().all(|component| {
        let Component::Normal(name) = component else {
            return false;
        };
        let Some(name) = name.to_str() else {
            return false;
        };
        let name = name.to_ascii_lowercase();
        !matches!(
            name.as_str(),
            ".git"
                | ".pi"
                | ".ssh"
                | ".aws"
                | ".gnupg"
                | ".config"
                | ".local"
                | ".env"
                | ".envrc"
                | ".npmrc"
                | ".pypirc"
                | ".netrc"
                | "auth.json"
                | "credentials"
                | "secrets"
        ) && !name.starts_with(".env.")
            && !name.starts_with("credentials.")
            && !name.starts_with("secrets.")
            && !name.starts_with("id_rsa")
            && !name.starts_with("id_ed25519")
            && ![".pem", ".key", ".p12", ".pfx"]
                .iter()
                .any(|suffix| name.ends_with(suffix))
    })
}

pub(crate) fn private_roots() -> Result<Vec<PathBuf>, String> {
    let roots =
        crate::config::Roots::from_env().map_err(|_| "Pi private paths could not be resolved")?;
    let mut paths = vec![roots.config, roots.data, roots.state, roots.cache];
    if let Some(path) = std::env::var_os("PI_CODING_AGENT_DIR") {
        paths.push(PathBuf::from(path));
    }
    if let Some(home) = std::env::var_os("HOME") {
        paths.push(PathBuf::from(home).join(".pi"));
    }
    let mut protected = Vec::new();
    for path in paths {
        if !path.is_absolute() {
            return Err("Pi private directory must be absolute".into());
        }
        if let Ok(canonical) = path.canonicalize()
            && canonical != path
        {
            protected.push(canonical);
        }
        protected.push(path);
    }
    Ok(protected)
}
