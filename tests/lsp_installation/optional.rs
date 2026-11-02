use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use rev::lsp::install::{self, Registry, State, Storage};
use rev::org::Cancellation;

use super::storage::roots;
use crate::fixture::Fixture;

pub(super) fn executable(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

fn sha(path: &Path) -> String {
    let output = Command::new("/usr/bin/sha256sum")
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .into()
}

pub(super) fn fixtures(directory: &Path) {
    fs::create_dir_all(directory.join("bin")).unwrap();
    for name in ["curl", "ruby", "gem", "python3"] {
        executable(
            &directory.join("bin").join(name),
            include_str!("optional-tools.sh"),
        );
    }
    let peer = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lsp/candidates.sh");
    executable(
        &directory.join("server"),
        &format!(
            r#"#!/bin/sh
set -eu
if test "${{1-}}" = --version; then echo 'controlled native server 1'; exit 0; fi
if test -f "$REV_OPTIONAL_FIXTURE/reject-probe"; then exit 45; fi
if test "${{0##*/}}" = ruby-lsp; then
    test "$BUNDLE_FROZEN" = true
    test "$GEM_HOME" = "${{0%/bin/ruby-lsp}}/gems"
    test "$BUNDLE_GEMFILE" = "${{0%/bin/ruby-lsp}}/Gemfile"
fi
exec /bin/sh '{}' normal "$REV_OPTIONAL_FIXTURE/probe"
"#,
            peer.display()
        ),
    );
    fs::write(directory.join("ruby.json"), serde_json::json!({"version":"0.26.1","bundler_version":"4.0.23","sha":sha(&write(directory,"gem-content","fixture-gem"))}).to_string()).unwrap();
    fs::write(directory.join("pip-report.json"), serde_json::json!({"install":[{"metadata":{"name":"python-lsp-server","version":"1.13.0"},"download_info":{"url":"https://files.pythonhosted.org/fixture.whl","archive_info":{"hashes":{"sha256":"a".repeat(64)}}}}]}).to_string()).unwrap();
    fs::write(directory.join("release.json"), serde_json::json!({"tag_name":"fixture-1","draft":false,"prerelease":false,"assets":[{"name":"marksman-linux-x64","size":fs::metadata(directory.join("server")).unwrap().len(),"digest":format!("sha256:{}",sha(&directory.join("server"))),"browser_download_url":"https://github.com/artempyanykh/marksman/releases/download/fixture-1/marksman-linux-x64"}]}).to_string()).unwrap();
}

fn write(directory: &Path, name: &str, text: &str) -> PathBuf {
    let path = directory.join(name);
    fs::write(&path, text).unwrap();
    path
}

pub(super) fn child(name: &str) -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("REV_OPTIONAL_FIXTURE") {
        return Some(root.into());
    }
    let f = Fixture::new();
    fixtures(&f.0);
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("REV_OPTIONAL_FIXTURE", &f.0)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", f.0.join("bin").display()),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    None
}

// Real filesystem, SHA-256, registry and transport boundaries wrap controlled
// tools. Failed acquisitions must retain the earlier ready payload verbatim.
#[test]
fn optional_acquisition() {
    let Some(directory) = child("optional::optional_acquisition") else {
        return;
    };
    for (recipe, language) in [
        ("marksman", "markdown"),
        ("ruby-lsp", "ruby"),
        ("python-lsp-server", "python"),
    ] {
        let f = Fixture::new();
        let roots = roots(&f);
        let mut storage = Storage::acquire(roots.clone()).unwrap();
        let acquire = |storage: &mut Storage| {
            let mut inherited = BTreeMap::new();
            if recipe == "ruby-lsp" {
                let mut server =
                    rev::config::Snapshot::default().settings().lsp.servers["rust"].clone();
                server.env = BTreeMap::from([
                    ("TMPDIR".into(), "/outside/tmp".into()),
                    ("MAKEFLAGS".into(), "-j999".into()),
                    ("RUBYOPT".into(), "invalid-option".into()),
                ]);
                inherited.insert("ruby".into(), server);
            }
            install::install(
                storage,
                recipe,
                language,
                inherited,
                &Cancellation::default(),
            )
        };
        let outcome = acquire(&mut storage).unwrap();
        assert_eq!(outcome.installation.status, State::Ready);
        assert!(outcome.selection.is_none());
        assert!(!roots.config.exists());
        let payload = roots.data.join(&outcome.installation.payload);
        let launch = &outcome.installation.launches[0];
        let before = fs::read(payload.join(&launch.executable)).unwrap();
        if recipe == "python-lsp-server" {
            assert_eq!(
                fs::read_to_string(directory.join("venvs")).unwrap().trim(),
                payload.join("venv").to_str().unwrap()
            );
        }
        if recipe == "ruby-lsp" {
            assert_eq!(
                launch.env["GEM_HOME"],
                payload.join("gems").to_str().unwrap()
            );
        }
        for (failure, expected) in [
            ("reject-tool", "helper exited"),
            ("reject-probe", "probe failed"),
            ("flood-stderr", "helper exited"),
        ] {
            fs::write(directory.join(failure), "").unwrap();
            let error = acquire(&mut storage).err().unwrap();
            assert!(error.contains(expected), "{recipe}: {error}");
            if failure == "flood-stderr" {
                assert!(
                    error.len() < install::DISPLAY_BYTES + 512,
                    "diagnostic tail exceeds byte ceiling"
                );
            }
            fs::remove_file(directory.join(failure)).unwrap();
            let registry = Registry::load(&roots.data).unwrap();
            assert_eq!(registry.installations[0], outcome.installation);
            assert_eq!(
                registry
                    .installations
                    .iter()
                    .filter(|i| i.status == State::Ready)
                    .count(),
                1
            );
            assert_eq!(fs::read(payload.join(&launch.executable)).unwrap(), before);
            assert!(!roots.config.exists());
            assert!(
                fs::read_dir(roots.cache.join("lsp"))
                    .unwrap()
                    .next()
                    .is_none()
            );
        }
        if recipe != "marksman" {
            fs::write(directory.join("reject-evidence"), "").unwrap();
            let error = acquire(&mut storage).err().unwrap();
            assert!(
                error.contains("differs") || error.contains("integrity"),
                "{error}"
            );
            fs::remove_file(directory.join("reject-evidence")).unwrap();
        }
    }
}

// Package metadata is rejected before a ready record can be published; a release
// hash, exact dependency versions and a base-only pip report are distinct facts.
#[test]
fn optional_evidence() {
    let Some(directory) = child("optional::optional_evidence") else {
        return;
    };
    for (recipe, language, file, field, value) in [
        (
            "marksman",
            "markdown",
            "release.json",
            "digest",
            serde_json::Value::Null,
        ),
        (
            "ruby-lsp",
            "ruby",
            "ruby.json",
            "version",
            serde_json::json!("../floating"),
        ),
        (
            "python-lsp-server",
            "python",
            "pip-report.json",
            "requested_extras",
            serde_json::json!(["all"]),
        ),
        (
            "marksman",
            "markdown",
            "release.json",
            "digest",
            serde_json::json!(format!("sha256:{}", "0".repeat(64))),
        ),
        (
            "ruby-lsp",
            "ruby",
            "ruby.json",
            "sha",
            serde_json::json!("missing"),
        ),
        (
            "python-lsp-server",
            "python",
            "pip-report.json",
            "url",
            serde_json::json!("https://other.example/fixture.whl"),
        ),
        (
            "python-lsp-server",
            "python",
            "pip-report.json",
            "sha256",
            serde_json::json!("missing"),
        ),
        (
            "python-lsp-server",
            "python",
            "pip-report.json",
            "version",
            serde_json::json!("2.0.0"),
        ),
    ] {
        let path = directory.join(file);
        let saved = fs::read(&path).unwrap();
        let mut json: serde_json::Value = serde_json::from_slice(&saved).unwrap();
        match recipe {
            "marksman" => json["assets"][0][field] = value,
            "python-lsp-server" => match field {
                "url" => json["install"][0]["download_info"][field] = value,
                "sha256" => {
                    json["install"][0]["download_info"]["archive_info"]["hashes"][field] = value
                }
                "version" => json["install"][0]["metadata"][field] = value,
                _ => json["install"][0][field] = value,
            },
            _ => json[field] = value,
        }
        fs::write(&path, json.to_string()).unwrap();
        let f = Fixture::new();
        let roots = roots(&f);
        let mut storage = Storage::acquire(roots.clone()).unwrap();
        assert!(
            install::install(
                &mut storage,
                recipe,
                language,
                BTreeMap::new(),
                &Cancellation::default()
            )
            .is_err()
        );
        assert!(
            Registry::load(&roots.data)
                .unwrap()
                .installations
                .iter()
                .all(|i| i.status != State::Ready)
        );
        fs::write(path, saved).unwrap();
    }
}

// Optional tools are consulted only for explicit acquisition. Missing runtimes
// must leave configuration, registry facts and disposable caches untouched.
#[test]
fn optional_prerequisites() {
    if std::env::var_os("REV_OPTIONAL_MISSING").is_none() {
        let f = Fixture::new();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "optional::optional_prerequisites", "--nocapture"])
            .env("REV_OPTIONAL_MISSING", "1")
            .env("PATH", &f.0)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    for (recipe, language, tool) in [
        ("marksman", "markdown", "curl"),
        ("ruby-lsp", "ruby", "ruby"),
        ("python-lsp-server", "python", "python3"),
        ("ols", "odin", "git"),
    ] {
        let f = Fixture::new();
        let roots = roots(&f);
        let mut storage = Storage::acquire(roots.clone()).unwrap();
        let error = install::install(
            &mut storage,
            recipe,
            language,
            BTreeMap::new(),
            &Cancellation::default(),
        )
        .err()
        .unwrap();
        assert!(
            error.contains("missing prerequisite") && error.contains(tool),
            "{error}"
        );
        assert!(
            Registry::load(&roots.data)
                .unwrap()
                .installations
                .is_empty()
        );
        assert!(!roots.config.exists());
        assert!(!roots.cache.exists());
    }
}

// Admission of persisted ready facts protects reuse as well as fresh setup.
// Corrupt Ruby/OLS runtime paths must not become authority to use outside data.
#[test]
fn runtime_admission() {
    let Some(_directory) = child("optional::runtime_admission") else {
        return;
    };
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let outcome = install::install(
        &mut storage,
        "ruby-lsp",
        "ruby",
        BTreeMap::new(),
        &Cancellation::default(),
    )
    .unwrap();
    drop(storage);
    let path = roots.data.join("lsp/installations.toml");
    let saved = fs::read_to_string(&path).unwrap();
    let home = &outcome.installation.launches[0].env["GEM_HOME"];
    fs::write(&path, saved.replace(home, "/outside/gems")).unwrap();
    assert!(
        Registry::load(&roots.data)
            .err()
            .unwrap()
            .contains("isolated runtime")
    );
    fs::write(&path, saved).unwrap();
    assert_eq!(
        Registry::load(&roots.data).unwrap().installations[0],
        outcome.installation
    );
}
