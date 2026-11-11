use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;
use std::process::Command;

use rev::lsp::install::{Registry, State, Storage, install_npm};
use rev::org::Cancellation;

use super::storage::roots;
use crate::fixture::Fixture;

const TS: &str = "typescript-language-server";
const WEB: &str = "vscode-langservers-extracted";

fn executable(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

pub(super) fn fixtures(root: &Path) {
    fs::create_dir_all(root.join("bin")).unwrap();
    executable(
        &root.join("bin/node"),
        r#"#!/bin/sh
set -eu
test -z "$NODE_OPTIONS"
if test "${1-}" = --version; then printf 'v26.8.1\n'; exit 0; fi
test "${1-}" = --input-type=commonjs
test "${2-}" = --eval
test -n "${3-}"
test -f "${4-}"
/usr/bin/sha256sum "$4" | /usr/bin/cut -d ' ' -f 1
"#,
    );
    executable(
        &root.join("bin/npm"),
        r#"#!/bin/sh
set -eu
prefix=''; scripts=0; exact=0; lock=0; registry=0; engine=0; global=0
for argument in "$@"; do
    case "$argument" in
        --prefix=*) prefix=${argument#--prefix=} ;;
        --ignore-scripts) scripts=1 ;;
        --save-exact) exact=1 ;;
        --package-lock=true) lock=1 ;;
        --registry=https://registry.npmjs.org) registry=1 ;;
        --engine-strict) engine=1 ;;
        --global=false) global=1 ;;
        --userconfig=*|--globalconfig=*) test -f "${argument#*=}" ;;
    esac
done
test "$scripts$exact$lock$registry$engine$global" = 111111
test "$prefix" = "$PWD"
printf '%s\n' "$*" >> "$REV_NPM_FIXTURE/calls"
while test "$#" -gt 0; do
    case "$1" in --version) printf '12.0.2\n'; exit 0 ;; --*) shift ;; *) break ;; esac
done
case "$1 $2" in
    'view typescript-language-server@latest') printf '"6.0.2"' ;;
    'view typescript@6')
        if test -f "$REV_NPM_FIXTURE/no-six"; then printf '["7.0.0","6.1.0-beta"]'; else printf '["6.0.2","6.0.3","7.0.0","6.1.0-beta","06.0.9"]'; fi ;;
    'view vscode-langservers-extracted@latest') printf '"4.10.0"' ;;
    'install typescript-language-server@6.0.2')
        test "${3-}" = typescript@6.0.3
        if test -f "$REV_NPM_FIXTURE/engine"; then printf 'unsupported engine' >&2; exit 1; fi
        /bin/cp -R "$REV_NPM_FIXTURE/ts/node_modules" "$prefix/node_modules"
        /bin/cp "$REV_NPM_FIXTURE/ts-lock.json" "$prefix/package-lock.json" ;;
    'install vscode-langservers-extracted@4.10.0')
        test "$#" = 2
        /bin/cp -R "$REV_NPM_FIXTURE/web/node_modules" "$prefix/node_modules"
        /bin/cp "$REV_NPM_FIXTURE/web-lock.json" "$prefix/package-lock.json" ;;
    *) exit 89 ;;
esac
"#,
    );
    package(root, "ts", TS, "6.0.2", &[TS]);
    package(root, "ts", "typescript", "6.0.3", &[]);
    package(
        root,
        "web",
        WEB,
        "4.10.0",
        &[
            "vscode-html-language-server",
            "vscode-css-language-server",
            "vscode-json-language-server",
        ],
    );
    lock(root, "ts", &[(TS, "6.0.2"), ("typescript", "6.0.3")]);
    lock(root, "web", &[(WEB, "4.10.0")]);
}

fn package(root: &Path, tree: &str, name: &str, version: &str, launches: &[&str]) {
    let module = root.join(tree).join("node_modules").join(name);
    fs::create_dir_all(module.join("bin")).unwrap();
    fs::write(
        module.join("package.json"),
        serde_json::json!({"name":name,"version":version}).to_string(),
    )
    .unwrap();
    let bins = root.join(tree).join("node_modules/.bin");
    fs::create_dir_all(&bins).unwrap();
    let peer = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lsp/candidates.sh");
    for launch in launches {
        executable(
            &module.join("bin").join(launch),
            &format!(
                r#"#!/bin/sh
set -eu
test "${{1-}}" = --stdio
printf '%s\n' '{launch}' >> "$REV_NPM_FIXTURE/probes"
/bin/ls -A >> "$REV_NPM_FIXTURE/probe-files"
if test -f "$REV_NPM_FIXTURE/probe-fail"; then exit 1; fi
exec /bin/sh '{}' normal "$REV_NPM_FIXTURE/{launch}"
"#,
                peer.display()
            ),
        );
        symlink(format!("../{name}/bin/{launch}"), bins.join(launch)).unwrap();
    }
}

fn lock(root: &Path, tree: &str, pins: &[(&str, &str)]) {
    let dependencies: BTreeMap<_, _> = pins.iter().copied().collect();
    let mut packages = serde_json::Map::new();
    packages.insert("".into(), serde_json::json!({"dependencies":dependencies}));
    for (name, version) in pins {
        packages.insert(format!("node_modules/{name}"),serde_json::json!({
            "version":version,"resolved":format!("https://registry.npmjs.org/{name}/-/package-{version}.tgz"),
            "integrity":format!("sha512-{}==","A".repeat(86))
        }));
    }
    fs::write(
        root.join(format!("{tree}-lock.json")),
        serde_json::json!({"lockfileVersion":3,"packages":packages}).to_string(),
    )
    .unwrap();
}

// Owned npm/Node fixtures assert recipe argv, exact TypeScript-major selection,
// package/lock admission and all three independent web probes. Real sha256sum
// hashes the lock, but only live smoke evidence establishes Node/npm behavior.
#[test]
fn isolated_packages() {
    let Some(root) = std::env::var_os("REV_NPM_FIXTURE") else {
        let f = Fixture::new();
        fixtures(&f.0);
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "npm::isolated_packages", "--nocapture"])
            .env("REV_NPM_FIXTURE", &f.0)
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", f.0.join("bin").display()),
            )
            .env("NODE_OPTIONS", "--invalid-user-option")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    };
    let directory = std::path::PathBuf::from(root);
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let mut base = rev::config::Lsp::default().servers["rust"].clone();
    base.env.insert("EXAMPLE_OPTION".into(), "retained".into());
    let inherited = BTreeMap::from([("css".into(), base.clone())]);
    let mut ready = Vec::new();
    for (recipe, count) in [(TS, 1), (WEB, 3)] {
        let outcome = install_npm(
            &mut storage,
            recipe,
            inherited.clone(),
            &Cancellation::default(),
        )
        .unwrap();
        assert_eq!(outcome.installation.status, State::Ready);
        assert_eq!(outcome.installation.launches.len(), count);
        assert!(
            outcome
                .installation
                .evidence
                .starts_with("npm-lock:sha256:")
        );
        assert!(
            outcome
                .installation
                .runtime
                .contains("node v26.8.1; npm 12.0.2")
        );
        if recipe == TS {
            assert!(outcome.installation.runtime.contains("typescript@6.0.3"));
        }
        if recipe == WEB {
            let css = outcome
                .installation
                .launches
                .iter()
                .find(|p| p.id == "vscode-css-language-server")
                .unwrap();
            assert_eq!(css.env, base.env);
            assert_eq!(css.root_markers, base.root_markers);
        }
        assert!(
            roots
                .data
                .join(&outcome.installation.payload)
                .join("package-lock.json")
                .is_file()
        );
        ready.push(outcome.installation);
    }
    let probes = fs::read_to_string(directory.join("probes")).unwrap();
    assert_eq!(probes.lines().count(), 4);
    for name in [
        TS,
        "vscode-html-language-server",
        "vscode-css-language-server",
        "vscode-json-language-server",
    ] {
        let frames = fs::read_to_string(directory.join(format!("{name}.frames"))).unwrap();
        for method in ["initialize", "shutdown", "exit"] {
            assert!(frames.contains(method));
        }
        assert!(!frames.contains("textDocument/"));
    }
    assert!(
        fs::read_to_string(directory.join("probe-files"))
            .unwrap()
            .is_empty()
    );
    assert!(!roots.config.exists());
    failures(&directory, &mut storage);
    assert_eq!(
        &Registry::load(&roots.data).unwrap().installations[..2],
        ready.as_slice()
    );
    assert!(
        storage
            .registry()
            .installations
            .iter()
            .skip(2)
            .all(|i| i.status == State::Incomplete)
    );
    assert!(
        fs::read_dir(roots.cache.join("lsp"))
            .unwrap()
            .next()
            .is_none()
    );
}

fn failures(directory: &Path, storage: &mut Storage) {
    let lock_path = directory.join("ts-lock.json");
    let good = fs::read(&lock_path).unwrap();
    for (failure, expected) in [
        ("engine", "compatible Node/npm"),
        ("six", "required major"),
        ("pin", "differs from exact"),
        ("integrity", "integrity for every"),
        ("source", "registry HTTPS"),
        ("identity", "installed npm"),
        ("escape", "escape"),
        ("probe", "protocol probe failed"),
    ] {
        fs::write(&lock_path, &good).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&good).unwrap();
        let metadata = directory.join("ts/node_modules/typescript/package.json");
        let original = fs::read(&metadata).unwrap();
        corrupt_package(directory, &metadata, &mut value, failure);
        fs::write(&lock_path, value.to_string()).unwrap();
        let error = install_npm(storage, TS, BTreeMap::new(), &Cancellation::default())
            .err()
            .expect(failure);
        assert!(error.contains(expected), "{failure}: {error}");
        for name in ["engine", "no-six", "probe-fail"] {
            let _ = fs::remove_file(directory.join(name));
        }
        fs::write(&metadata, &original).unwrap();
        if failure == "escape" {
            let bin = directory.join("ts/node_modules/.bin").join(TS);
            fs::remove_file(&bin).unwrap();
            symlink(format!("../{TS}/bin/{TS}"), bin).unwrap();
        }
    }
}

// Both prerequisite failures precede any installation facts or payload creation.
#[test]
fn missing_runtime() {
    if std::env::var_os("REV_NPM_MISSING").is_none() {
        let f = Fixture::new();
        for prerequisite in ["node", "npm"] {
            if prerequisite == "npm" {
                executable(&f.0.join("node"), "#!/bin/sh\nexit 0\n");
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "npm::missing_runtime", "--nocapture"])
                .env("REV_NPM_MISSING", prerequisite)
                .env("PATH", &f.0)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    }
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let error = install_npm(&mut storage, TS, BTreeMap::new(), &Cancellation::default())
        .err()
        .unwrap();
    let missing = std::env::var("REV_NPM_MISSING").unwrap();
    assert!(
        error.contains("missing prerequisite") && error.contains(&missing),
        "{error}"
    );
    assert!(storage.registry().installations.is_empty());
    assert!(!roots.data.join("lsp/installations.toml").exists());
}

fn corrupt_package(
    directory: &Path,
    metadata: &Path,
    value: &mut serde_json::Value,
    failure: &str,
) {
    match failure {
        "engine" => {
            fs::write(directory.join("engine"), "").unwrap();
        }
        "six" => {
            fs::write(directory.join("no-six"), "").unwrap();
        }
        "pin" => {
            value["packages"][""]["dependencies"]["typescript"] = "7.0.0".into();
        }
        "integrity" => {
            value["packages"]["node_modules/typescript"]["integrity"] = "".into();
        }
        "source" => {
            value["packages"]["node_modules/typescript"]["resolved"] = "file:///outside".into();
        }
        "identity" => {
            fs::write(metadata, r#"{"name":"typescript","version":"7.0.0"}"#).unwrap();
        }
        "escape" => {
            let bin = directory.join("ts/node_modules/.bin").join(TS);
            fs::remove_file(&bin).unwrap();
            symlink("/bin/sh", bin).unwrap();
        }
        "probe" => {
            fs::write(directory.join("probe-fail"), "").unwrap();
        }
        _ => unreachable!(),
    }
}
