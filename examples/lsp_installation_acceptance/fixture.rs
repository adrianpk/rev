use crate::support::Result;
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

const URL: &str = "https://github.com/rust-lang/rust-analyzer/releases/download/acceptance-local/rust-analyzer-x86_64-unknown-linux-gnu.gz";

// Controlled acquisition transports the real installed binary. Its digest
// verifies these local bytes; it is not an upstream release attestation.
pub(crate) fn prepare(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("bin"))?;
    let asset = root.join("server.gz");
    let digest = compress(&asset)?;
    let release = json!({"tag_name":"acceptance-local","draft":false,"prerelease":false,"assets":[{"name":"rust-analyzer-x86_64-unknown-linux-gnu.gz","size":fs::metadata(&asset)?.len(),"digest":format!("sha256:{digest}"),"browser_download_url":URL}]});
    fs::write(root.join("release.json"), release.to_string())?;
    scripts(root)?;
    println!(
        "FIXTURE local Rust asset: compressed_bytes={} sha256={digest}; no network",
        fs::metadata(asset)?.len()
    );
    Ok(())
}

fn compress(asset: &Path) -> Result<String> {
    let analyzer =
        std::env::var_os("REV_ACCEPTANCE_RUST").ok_or("missing installed Rust server")?;
    let output = fs::File::create(asset)?;
    assert!(
        Command::new("gzip")
            .args(["--stdout", "--"])
            .arg(analyzer)
            .stdout(output)
            .status()?
            .success()
    );
    let digest = Command::new("sha256sum").arg(asset).output()?;
    assert!(digest.status.success());
    let digest = String::from_utf8(digest.stdout)?;
    Ok(digest
        .split_whitespace()
        .next()
        .ok_or("missing digest")?
        .to_owned())
}

fn scripts(root: &Path) -> Result<()> {
    let curl = root.join("bin/curl");
    fs::write(&curl, include_str!("tools.sh"))?;
    fs::set_permissions(curl, fs::Permissions::from_mode(0o700))?;
    fs::write(
        root.join("peer.sh"),
        include_str!("../../tests/lsp/candidates.sh"),
    )?;
    Ok(())
}
