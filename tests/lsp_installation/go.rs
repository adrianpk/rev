use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use rev::lsp::install::{Registry, State, Storage, install_go};
use rev::org::Cancellation;

use super::storage::roots;
use crate::fixture::Fixture;

const MODULE: &str = "golang.org/x/tools/gopls";
const SUM: &str = "h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

fn executable(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

pub(super) fn fixtures(root: &Path) {
    fs::create_dir_all(root.join("bin")).unwrap();
    executable(
        &root.join("bin/go"),
        r#"#!/bin/sh
set -eu
test "$GOTOOLCHAIN" = local
test "$GOENV" = off
test "$GOWORK" = off
test "$GOFLAGS" = '-modcacherw -p=2'
test "$GOSUMDB" = sum.golang.org
test "$GOPROXY" = https://proxy.golang.org
for directory in "$GOPATH" "$GOMODCACHE" "$GOCACHE" "$GOTMPDIR"; do
    case "$directory" in "$PWD"/*) test -d "$directory" ;; *) exit 81 ;; esac
done
printf '%s\n' "$*" >> "$REV_GO_FIXTURE/calls"
case "$*" in
    version) printf 'go version go1.27.1 linux/amd64\n' ;;
    'list -m -json golang.org/x/tools/gopls@latest')
        if test -f "$REV_GO_FIXTURE/compiler"; then printf 'requires go >= 9.0; GOTOOLCHAIN=local' >&2; exit 1; fi
        /bin/cat "$REV_GO_FIXTURE/module.json" ;;
    'mod download -json golang.org/x/tools/gopls@v0.23.0') /bin/cat "$REV_GO_FIXTURE/module.json" ;;
    'install -trimpath golang.org/x/tools/gopls@v0.23.0')
        case "$GOBIN" in */lsp/installations/gopls/*) ;; *) exit 82 ;; esac
        /bin/cp "$REV_GO_FIXTURE/server" "$GOBIN/gopls" ;;
    'version -m -json '*) /bin/cat "$REV_GO_FIXTURE/build.json" ;;
    *) exit 83 ;;
esac
"#,
    );
    let peer = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lsp/candidates.sh");
    executable(
        &root.join("server"),
        &format!(
            r#"#!/bin/sh
set -eu
if test "${{1-}}" = version; then printf 'golang.org/x/tools/gopls v0.23.0\n'; exit 0; fi
printf '%s\n' "$PWD" >> "$REV_GO_FIXTURE/probe-cwd"
/bin/ls -A >> "$REV_GO_FIXTURE/probe-files"
if test -f "$REV_GO_FIXTURE/probe-fail"; then exit 1; fi
exec /bin/sh '{}' normal "$REV_GO_FIXTURE/probe"
"#,
            peer.display()
        ),
    );
    metadata(root, "v0.23.0", SUM, false);
}

fn metadata(root: &Path, version: &str, sum: &str, replaced: bool) {
    let mut module = serde_json::json!({"Path":MODULE,"Version":version,"Sum":sum,"GoModSum":SUM});
    if replaced {
        module["Replace"] = serde_json::json!({"Path":"example.test/other"});
    }
    fs::write(root.join("module.json"), module.to_string()).unwrap();
    fs::write(root.join("build.json"),serde_json::json!({"Path":MODULE,"Main":{"Path":MODULE,"Version":"v0.23.0","Sum":SUM},"GoVersion":"go1.27.1"}).to_string()).unwrap();
}

// A controlled Go boundary verifies structured exact module operations, local
// compiler/caches, build identity and empty-root probes. It cannot establish
// upstream Go package or semantic quality; live evidence is recorded separately.
#[test]
fn isolated_module() {
    let Some(root) = std::env::var_os("REV_GO_FIXTURE") else {
        let f = Fixture::new();
        fixtures(&f.0);
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "go::isolated_module", "--nocapture"])
            .env("REV_GO_FIXTURE", &f.0)
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", f.0.join("bin").display()),
            )
            .env("GOTOOLCHAIN", "auto")
            .env("GOFLAGS", "-invalid-user-flag")
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
    let mut inherited = rev::config::Lsp::default().servers["rust"].clone();
    inherited.languages = vec!["go".into()];
    inherited.command = "custom-gopls".into();
    inherited.root_markers = vec!["go.mod".into(), "custom.work".into()];
    inherited
        .env
        .insert("EXAMPLE_OPTION".into(), "retained".into());
    inherited.settings = toml::toml! { custom = true }.into();
    let outcome = install_go(
        &mut storage,
        Some(inherited.clone()),
        &Cancellation::default(),
    )
    .unwrap();
    assert_eq!(outcome.installation.status, State::Ready);
    assert_eq!(outcome.installation.version, "v0.23.0");
    assert_eq!(outcome.installation.evidence, format!("go-module:{SUM}"));
    assert_eq!(outcome.installation.launches[0].env, inherited.env);
    assert_eq!(
        outcome.installation.launches[0].settings,
        inherited.settings
    );
    assert_eq!(
        outcome.installation.launches[0].root_markers,
        inherited.root_markers
    );
    assert!(!roots.config.exists());
    let payload = roots.data.join(&outcome.installation.payload);
    assert!(payload.join("go-module.json").is_file());
    assert!(payload.join("go-build-info.json").is_file());
    assert!(
        fs::read_to_string(directory.join("probe-files"))
            .unwrap()
            .is_empty()
    );
    let frames = fs::read_to_string(directory.join("probe.frames")).unwrap();
    for method in ["initialize", "initialized", "shutdown", "exit"] {
        assert!(frames.contains(method));
    }
    assert!(!frames.contains("textDocument/"));
    for (failure, expected) in [
        ("compiler", "sufficient Go version"),
        ("version", "exact stable version"),
        ("sum", "checksum-database provenance"),
        ("replace", "unreplaced"),
        ("build", "differs from resolved module"),
        ("probe", "protocol probe failed"),
    ] {
        metadata(&directory, "v0.23.0", SUM, false);
        fail_module(&directory, failure);
        let error = install_go(&mut storage, None, &Cancellation::default())
            .err()
            .expect(failure);
        assert!(error.contains(expected), "{failure}: {error}");
        if failure == "compiler" {
            fs::remove_file(directory.join("compiler")).unwrap();
        }
        if failure == "probe" {
            fs::remove_file(directory.join("probe-fail")).unwrap();
        }
        assert_eq!(
            Registry::load(&roots.data).unwrap().installations[0],
            outcome.installation
        );
        assert!(
            fs::read_dir(roots.cache.join("lsp"))
                .unwrap()
                .next()
                .is_none()
        );
    }
    assert!(
        storage
            .registry()
            .installations
            .iter()
            .skip(1)
            .all(|i| i.status == State::Incomplete)
    );
    let calls = fs::read_to_string(directory.join("calls")).unwrap();
    assert_eq!(calls.matches("@latest").count(), 7);
    assert!(!calls.contains("env -w"));
}

fn fail_module(directory: &Path, failure: &str) {
    match failure {
        "compiler" => {
            fs::write(directory.join("compiler"), "").unwrap();
        }
        "version" => metadata(directory, "latest", SUM, false),
        "sum" => metadata(directory, "v0.23.0", "", false),
        "replace" => metadata(directory, "v0.23.0", SUM, true),
        "build" => {
            let p = directory.join("build.json");
            let s = fs::read_to_string(&p).unwrap();
            fs::write(
                p,
                s.replace(SUM, "h1:BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB="),
            )
            .unwrap();
        }
        "probe" => {
            fs::write(directory.join("probe-fail"), "").unwrap();
        }
        _ => unreachable!(),
    }
}

// Missing Go is diagnosed before version resolution or candidate publication.
#[test]
fn missing_compiler() {
    if std::env::var_os("REV_GO_MISSING").is_none() {
        let f = Fixture::new();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "go::missing_compiler"])
            .env("REV_GO_MISSING", "1")
            .env("PATH", &f.0)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        return;
    }
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let error = install_go(&mut storage, None, &Cancellation::default())
        .err()
        .unwrap();
    assert!(error.contains("missing prerequisite") && error.contains("go"));
    assert!(storage.registry().installations.is_empty());
    assert!(!roots.data.join("lsp/installations.toml").exists());
}
