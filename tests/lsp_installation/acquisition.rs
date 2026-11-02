use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use rev::lsp::install::{Registry, State, Storage, install_rust};
use rev::org::Cancellation;

use super::storage::roots;
use crate::fixture::Fixture;

const URL: &str = "https://github.com/rust-lang/rust-analyzer/releases/download/fixture-1/rust-analyzer-x86_64-unknown-linux-gnu.gz";

fn executable(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

fn metadata(directory: &Path, digest: Option<&str>) {
    let bytes = fs::metadata(directory.join("server.gz")).unwrap().len();
    let release = serde_json::json!({"tag_name":"fixture-1","draft":false,"prerelease":false,"assets":[{"name":"rust-analyzer-x86_64-unknown-linux-gnu.gz","size":bytes,"digest":digest,"browser_download_url":URL}]});
    fs::write(directory.join("release.json"), release.to_string()).unwrap();
}

fn digest(directory: &Path) -> String {
    let output = Command::new("/usr/bin/sha256sum")
        .arg(directory.join("server.gz"))
        .output()
        .unwrap();
    assert!(output.status.success());
    format!(
        "sha256:{}",
        String::from_utf8(output.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
    )
}

fn fixtures(directory: &Path) {
    fs::create_dir_all(directory.join("bin")).unwrap();
    executable(
        &directory.join("bin/curl"),
        r#"#!/bin/sh
set -eu
printf '%s\n' "$PWD" >> "$REV_RUST_FIXTURE/helper-cwd"
if test -f "$REV_RUST_FIXTURE/hold"; then
    trap '' TERM
    /bin/sh -c 'trap "" TERM; /bin/sleep 60' &
    printf '%s %s\n' "$$" "$!" > "$REV_RUST_FIXTURE/held-pids"
    wait
fi
if test -f "$REV_RUST_FIXTURE/flood"; then /usr/bin/head -c 1048577 /dev/zero; exit 0; fi
url=''; output=
while test "$#" -gt 0; do
    case "$1" in
        --url) shift; url=$1 ;;
        --output) shift; output=$1 ;;
    esac
    shift
done
case "$url" in
    https://api.github.com/repos/rust-lang/rust-analyzer/releases/latest) /bin/cat "$REV_RUST_FIXTURE/release.json" ;;
    https://github.com/rust-lang/rust-analyzer/releases/download/fixture-1/rust-analyzer-x86_64-unknown-linux-gnu.gz)
        test -n "$output"
        /bin/cp "$REV_RUST_FIXTURE/server.gz" "$output" ;;
    *) exit 88 ;;
esac
"#,
    );
    executable(
        &directory.join("bin/rustc"),
        "#!/bin/sh\nprintf '%s\\n' \"$REV_RUST_FIXTURE/missing-sysroot\"\n",
    );
    executable(&directory.join("bin/cargo"), "#!/bin/sh\nexit 0\n");
    let peer = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lsp/candidates.sh");
    let source = directory.join("server");
    executable(
        &source,
        &format!(
            r#"#!/bin/sh
set -eu
if test "${{1-}}" = --version; then printf 'rust-analyzer owned protocol fixture\n'; exit 0; fi
printf '%s\n' "$PWD" > "$REV_RUST_FIXTURE/probe-cwd"
/bin/ls -A > "$REV_RUST_FIXTURE/probe-initial-files"
if test -f "$REV_RUST_FIXTURE/reject-probe"; then printf 'invalid protocol\r\n\r\n'; exit 1; fi
exec /bin/sh '{}' normal "$REV_RUST_FIXTURE/probe"
"#,
            peer.display()
        ),
    );
    let compressed = Command::new("/usr/bin/gzip")
        .args(["--stdout", "--"])
        .arg(source)
        .output()
        .unwrap();
    assert!(compressed.status.success());
    fs::write(directory.join("server.gz"), compressed.stdout).unwrap();
    metadata(directory, Some(&digest(directory)));
}

// Actual curl/gzip/SHA-256 invocation boundaries use an owned curl fixture and
// the real gzip/digest tools. The payload is a controlled protocol peer; this
// proves acquisition/ownership mechanics, not upstream Rust semantic quality.
#[test]
fn verified_acquisition() {
    let Some(directory) = std::env::var_os("REV_RUST_FIXTURE") else {
        let f = Fixture::new();
        fixtures(&f.0);
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "acquisition::verified_acquisition",
                "--nocapture",
            ])
            .env("REV_RUST_FIXTURE", &f.0)
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
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let outcome = install_rust(&mut storage, None, &Cancellation::default()).unwrap();
    assert_eq!(outcome.installation.status, State::Ready);
    assert_eq!(outcome.installation.version, "fixture-1");
    assert_eq!(outcome.installation.source, URL);
    assert_eq!(outcome.installation.evidence, digest(&directory));
    assert!(outcome.selection.is_none());
    assert!(outcome.message.contains("Rust sources missing"));
    assert!(!roots.config.exists());
    let installed = roots
        .data
        .join(&outcome.installation.payload)
        .join("rust-analyzer");
    let original = fs::read(&installed).unwrap();
    let frames = fs::read_to_string(directory.join("probe.frames")).unwrap();
    for method in ["initialize", "initialized", "shutdown", "exit"] {
        assert!(
            frames.contains(&format!("\"method\":\"{method}\"")),
            "{method}"
        );
    }
    assert!(!frames.contains("textDocument/"));
    assert_eq!(
        fs::read_to_string(directory.join("probe-initial-files")).unwrap(),
        ""
    );
    assert!(
        fs::read_dir(roots.cache.join("lsp"))
            .unwrap()
            .next()
            .is_none()
    );
    let good = digest(&directory);
    for (failure, expected) in [
        ("checksum", "SHA-256 mismatch"),
        ("probe", "probe failed"),
        ("metadata", "integrity evidence"),
        ("oversize", "transfer size"),
        ("flood", "output exceeds 1 MiB"),
        ("gzip", "helper exited"),
    ] {
        match failure {
            "checksum" => metadata(&directory, Some(&format!("sha256:{}", "0".repeat(64)))),
            "probe" => {
                metadata(&directory, Some(&good));
                fs::write(directory.join("reject-probe"), "reject").unwrap();
            }
            "metadata" => metadata(&directory, None),
            "oversize" => {
                metadata(&directory, Some(&good));
                let path = directory.join("release.json");
                let mut value: serde_json::Value =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                value["assets"][0]["size"] =
                    serde_json::json!(rev::lsp::install::ARTIFACT_BYTES + 1);
                fs::write(path, value.to_string()).unwrap();
            }
            "flood" => {
                fs::write(directory.join("flood"), "flood").unwrap();
            }
            "gzip" => {
                fs::write(directory.join("server.gz"), "corrupt gzip").unwrap();
                metadata(&directory, Some(&digest(&directory)));
            }
            _ => unreachable!(),
        }
        let error = install_rust(&mut storage, None, &Cancellation::default())
            .err()
            .expect(failure);
        assert!(error.contains(expected), "{failure}: {error}");
        if failure == "flood" {
            fs::remove_file(directory.join("flood")).unwrap();
        }
        let registry = Registry::load(&roots.data).unwrap();
        assert_eq!(registry.installations[0], outcome.installation, "{failure}");
        assert_eq!(
            registry
                .installations
                .iter()
                .filter(|i| i.status == State::Ready)
                .count(),
            1,
            "{failure}"
        );
        assert_eq!(fs::read(&installed).unwrap(), original, "{failure}");
        assert!(!roots.config.exists());
        assert!(
            fs::read_dir(roots.cache.join("lsp"))
                .unwrap()
                .next()
                .is_none()
        );
    }
    assert_eq!(Registry::load(&roots.data).unwrap().installations.len(), 4);
}

// Optional acquisition tools are required only for explicit managed work. A
// missing prerequisite leaves configuration and installation facts untouched.
#[test]
fn missing_prerequisite() {
    if std::env::var_os("REV_RUST_MISSING").is_none() {
        let f = Fixture::new();
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "acquisition::missing_prerequisite",
                "--nocapture",
            ])
            .env("REV_RUST_MISSING", "1")
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
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let error = install_rust(&mut storage, None, &Cancellation::default())
        .err()
        .unwrap();
    assert!(
        error.contains("missing prerequisite") && error.contains("curl"),
        "{error}"
    );
    assert_eq!(Registry::load(&roots.data).unwrap().installations.len(), 0);
    assert!(!roots.config.exists());
    assert!(!roots.cache.exists());
}

// Cancellation must force/reap an owned leader that ignores TERM and stop its
// descendant group before freeing the native worker slot. No process-name
// matching or global environment mutation participates in this test.
#[test]
fn cancelled_acquisition() {
    if std::env::var_os("REV_RUST_FIXTURE").is_none() {
        let f = Fixture::new();
        fixtures(&f.0);
        fs::write(f.0.join("hold"), "hold").unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "acquisition::cancelled_acquisition",
                "--nocapture",
            ])
            .env("REV_RUST_FIXTURE", &f.0)
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
        return;
    }
    use rev::lsp::install::Manager;
    use std::time::{Duration, Instant};
    let directory = std::path::PathBuf::from(std::env::var_os("REV_RUST_FIXTURE").unwrap());
    let f = Fixture::new();
    let roots = roots(&f);
    let buffers = rev::buffer::Buffers::new().unwrap();
    let source = buffers.get(buffers.initial()).unwrap().stamp();
    let mut manager = Manager::default();
    manager
        .start(source, roots.clone(), |mut storage, cancel| {
            install_rust(&mut storage, None, cancel)
        })
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let pid_file = directory.join("held-pids");
    while !pid_file.exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    let pids: Vec<u32> = fs::read_to_string(pid_file)
        .unwrap()
        .split_whitespace()
        .map(|p| p.parse().unwrap())
        .collect();
    assert_eq!(pids.len(), 2);
    assert!(manager.cancel());
    assert!(manager.active());
    assert!(
        manager
            .start(source, roots.clone(), |_, _| unreachable!())
            .is_err()
    );
    loop {
        assert!(Instant::now() < deadline);
        if let Some(result) = manager.poll() {
            assert!(result.err().unwrap().contains("cancelled"));
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!manager.active());
    assert!(
        !Path::new(&format!("/proc/{}", pids[0])).exists(),
        "leader not reaped"
    );
    for pid in pids {
        if let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) {
            let state = stat
                .rsplit_once(')')
                .unwrap()
                .1
                .split_whitespace()
                .next()
                .unwrap();
            assert!(
                matches!(state, "Z" | "X" | "x"),
                "owned descendant still running: {pid} {state}"
            );
        }
    }
    assert_eq!(Registry::load(&roots.data).unwrap().installations.len(), 0);
    assert!(
        fs::read_dir(roots.cache.join("lsp"))
            .unwrap()
            .next()
            .is_none()
    );
}
