//! Shared bounded official-release metadata and HTTPS invocation for native
//! recipes. Missing upstream digests never become locally invented evidence.
use super::helper::Helper;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset {
    name: String,
    digest: Option<String>,
    browser_download_url: String,
    size: u64,
}
pub(super) struct Exact {
    pub version: String,
    pub url: String,
    pub digest: String,
    pub bytes: u64,
}

pub(super) fn release(bytes: &[u8], repository: &str, name: &str) -> Result<Exact, String> {
    let release: Release =
        serde_json::from_slice(bytes).map_err(|_| "invalid official release metadata")?;
    if release.draft || release.prerelease || !tag(&release.tag_name) {
        return Err("official release lacks an admitted exact stable tag".into());
    }
    let asset = release
        .assets
        .into_iter()
        .find(|a| a.name == name)
        .ok_or("official release lacks the verified Linux x86-64 asset")?;
    exact(&release.tag_name, repository, name, &asset)
}

fn exact(tag: &str, repository: &str, name: &str, asset: &Asset) -> Result<Exact, String> {
    let url = format!(
        "https://github.com/{repository}/releases/download/{}/{name}",
        tag
    );
    let digest = asset
        .digest
        .as_deref()
        .and_then(|s| s.strip_prefix("sha256:"))
        .ok_or("official asset lacks SHA-256 integrity evidence")?;
    if asset.browser_download_url != url || asset.size == 0 || asset.size > super::ARTIFACT_BYTES {
        return Err("official asset has an invalid source or transfer size".into());
    }
    if !super::evidence::sha256(digest) {
        return Err("official asset has an invalid SHA-256 digest".into());
    }
    Ok(Exact {
        version: tag.into(),
        url,
        digest: digest.to_ascii_lowercase(),
        bytes: asset.size,
    })
}

fn tag(tag: &str) -> bool {
    tag.len() <= 128
        && tag
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && tag
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

pub(super) fn curl(
    tool: &Path,
    url: &str,
    directory: &Path,
    cache: &Path,
    output: Option<&Path>,
) -> Helper {
    let mut args = [
        "-q",
        "--fail",
        "--silent",
        "--show-error",
        "--location",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--max-time",
        "180",
        "--max-filesize",
        "268435456",
        "--url",
        url,
    ]
    .map(str::to_owned)
    .to_vec();
    if let Some(output) = output {
        args.extend(["--output".into(), output.to_string_lossy().into_owned()]);
    }
    Helper {
        executable: tool.to_owned(),
        args,
        directory: directory.to_owned(),
        env: Default::default(),
        seconds: 185,
        watch: vec![(cache.to_owned(), super::CACHE_BYTES)],
    }
}
