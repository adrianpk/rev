//! Cargo selects one SDK dependency graph; directory timestamps cannot identify it.
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf, process::Command, sync::OnceLock};

pub(super) struct Artifacts {
    pub sdk: PathBuf,
    pub json: PathBuf,
}

pub(super) fn get() -> &'static Artifacts {
    static ARTIFACTS: OnceLock<Artifacts> = OnceLock::new();
    ARTIFACTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // This worktree-local cache avoids the enclosing Cargo test's target
        // lock and reuses the small SDK build across fixture processes/runs.
        let output = Command::new(env!("CARGO"))
            .current_dir(&root)
            .args([
                "build",
                "--locked",
                "--package",
                "rev-plugin-sdk",
                "--jobs",
                "1",
                "--message-format=json-render-diagnostics",
                "--target-dir",
            ])
            .arg(root.join("target/plugin-peer-sdk"))
            .output()
            .expect("build the public SDK fixture dependencies");
        assert!(
            output.status.success(),
            "SDK fixture build: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut libraries = BTreeMap::new();
        for line in output
            .stdout
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
        {
            let message: Value = serde_json::from_slice(line).expect("Cargo artifact message");
            if message["reason"] != "compiler-artifact" {
                continue;
            }
            let name = message["target"]["name"]
                .as_str()
                .expect("Cargo target name");
            if !["rev_plugin_sdk", "serde_json"].contains(&name) {
                continue;
            }
            let library = message["filenames"]
                .as_array()
                .expect("Cargo artifact filenames")
                .iter()
                .filter_map(Value::as_str)
                .map(PathBuf::from)
                .find(|path| path.extension().is_some_and(|ext| ext == "rlib"))
                .expect("Cargo-built Rust library");
            assert!(library.is_file(), "missing Cargo artifact: {library:?}");
            assert!(
                libraries.insert(name.to_owned(), library).is_none(),
                "duplicate Cargo library: {name}"
            );
        }
        let sdk = libraries
            .remove("rev_plugin_sdk")
            .expect("public SDK artifact");
        let json = libraries.remove("serde_json").expect("SDK JSON artifact");
        Artifacts { sdk, json }
    })
}
