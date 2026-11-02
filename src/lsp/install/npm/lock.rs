use std::path::Path;

use super::super::package;
use super::Pin;

pub(super) fn admit(bytes: &[u8], pins: &[Pin]) -> Result<(), String> {
    let lock: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| "invalid npm package lock")?;
    if !matches!(lock["lockfileVersion"].as_u64(), Some(2 | 3)) {
        return Err("npm requires a version-2/3 package lock with integrity records".into());
    }
    let packages = lock["packages"]
        .as_object()
        .ok_or("npm lock lacks package records")?;
    for pin in pins {
        if packages
            .get("")
            .and_then(|v| v["dependencies"][pin.name].as_str())
            != Some(&pin.version)
        {
            return Err(format!("npm lock differs from exact {} version", pin.name));
        }
        let key = format!("node_modules/{}", pin.name);
        if packages.get(&key).and_then(|v| v["version"].as_str()) != Some(&pin.version) {
            return Err(format!("npm lock lacks the resolved {} package", pin.name));
        }
    }
    for (name, package) in packages.iter().filter(|(name, _)| !name.is_empty()) {
        provenance(name, package)?;
    }
    Ok(())
}

fn provenance(name: &str, package: &serde_json::Value) -> Result<(), String> {
    let url = package["resolved"].as_str().unwrap_or("");
    let integrity = package["integrity"].as_str().unwrap_or("");
    if !name.starts_with("node_modules/")
        || package["link"] == true
        || !url.starts_with("https://registry.npmjs.org/")
        || !integrity_valid(integrity)
    {
        return Err(
            "npm lock requires registry HTTPS sources and SHA-256/512 integrity for every package"
                .into(),
        );
    }
    if !package["version"]
        .as_str()
        .is_some_and(super::version::exact)
    {
        return Err("npm lock requires exact dependency versions".into());
    }
    Ok(())
}

fn integrity_valid(value: &str) -> bool {
    if value.is_empty() || value.len() > 512 {
        return false;
    }
    value.split_whitespace().all(|s| {
        if let Some(digest) = s.strip_prefix("sha512-") {
            return base64(digest, 88, 2);
        }
        s.strip_prefix("sha256-")
            .is_some_and(|digest| base64(digest, 44, 1))
    })
}

fn base64(digest: &str, length: usize, padding: usize) -> bool {
    digest.len() == length
        && digest.ends_with(&"=".repeat(padding))
        && digest.as_bytes()[..length - padding]
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || b"+/".contains(b))
}

pub(super) fn installed(payload: &Path, pin: &Pin) -> Result<(), String> {
    let bytes = package::read(payload, &format!("node_modules/{}/package.json", pin.name))?;
    let package: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "invalid installed npm package metadata")?;
    if package["name"] != pin.name || package["version"] != pin.version {
        return Err(format!(
            "installed npm {} differs from resolved package identity",
            pin.name
        ));
    }
    Ok(())
}
