use super::push::{object, prepare, tracking};
use super::*;
use std::{
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};

fn executable(path: &std::path::Path, text: &str) {
    fs::write(path, text).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}
fn ready(repository: &Repository) -> PathBuf {
    repository.commit();
    let origin = remote(repository, "origin");
    tracking(repository);
    repository.commit();
    origin
}
fn wait_path(path: &std::path::Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(Instant::now() < deadline, "Owned helper did not start");
        std::thread::sleep(Duration::from_millis(2));
    }
}

// A real configuration edit after the first private read invalidates fetch
// before its changing child; the request never adopts a replacement refspec.
#[test]
fn changed_fetch_config() {
    let repository = Repository::new("sha1");
    ready(&repository);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&mut owner);
    let tracking_before = repository.git(["rev-parse", "refs/remotes/origin/main"]);
    executable(
        &repository.program,
        r#"#!/bin/sh
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
if [ "$2" = fetch ]; then printf 'fetch\n' >> "$0.fetch"; fi
if [ "$2" = config ] && [ "$3" = --null ] && [ "$4" = --list ] && [ ! -f "$0.changed" ]; then
    /usr/bin/git "$@" || exit "$?"
    /usr/bin/git config remote.origin.fetch 'refs/heads/*:refs/remotes/origin/*' || exit "$?"
    : > "$0.changed"
    exit 0
fi
exec /usr/bin/git "$@"
"#,
    );
    let id = owner.fetch(refs, b"origin").unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.effects, Effects::NotStarted);
    assert_eq!(record.outcome, Outcome::Failed);
    assert!(
        record
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("configuration changed")
    );
    assert!(!repository.program.with_extension("fetch").exists());
    assert_eq!(
        repository.git(["rev-parse", "refs/remotes/origin/main"]),
        tracking_before
    );
}

// The actual push succeeds, then its owned bare endpoint becomes unavailable
// before real ls-remote. Provenance/exit zero cannot stand in for verification.
#[test]
fn unavailable_verification() {
    let repository = Repository::new("sha256");
    let origin = ready(&repository);
    let tip = repository.git(["rev-parse", "HEAD"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&mut owner);
    let intent = prepare(&mut owner, refs);
    fs::write(
        repository.program.with_extension("endpoint"),
        origin.to_str().unwrap(),
    )
    .unwrap();
    executable(
        &repository.program,
        r#"#!/bin/sh
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
if [ "$2" = push ]; then
    printf 'push\n' >> "$0.pushes"
    /usr/bin/git "$@" || exit "$?"
    endpoint=$(cat "$0.endpoint")
    mv "$endpoint" "$endpoint.offline" || exit "$?"
    exit 0
fi
exec /usr/bin/git "$@"
"#,
    );
    let id = owner.push(intent).unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::Indeterminate, "{record:?}");
    assert_eq!(record.effects, Effects::Possible);
    assert!(record.exit.unwrap().success());
    assert!(record.argv.iter().any(|v| v == "push"));
    assert!(!format!("{record:?}").contains(origin.to_str().unwrap()));
    assert_eq!(
        fs::read(repository.program.with_extension("pushes")).unwrap(),
        b"push\n"
    );
    assert!(!owner.busy());
    let offline = PathBuf::from(format!("{}.offline", origin.display()));
    fs::rename(offline, &origin).unwrap();
    assert_eq!(object(&repository, &origin, "main"), tip);
}

fn helper(repository: &Repository, body: &str) -> PathBuf {
    let directory = repository.fixture.0.join("helpers");
    fs::create_dir_all(&directory).unwrap();
    let helper = directory.join("git-remote-owned");
    executable(&helper, body);
    executable(
        &repository.program,
        r#"#!/bin/sh
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
unset GIT_ASKPASS SSH_ASKPASS GIT_CONFIG_COUNT GIT_CONFIG_PARAMETERS
PATH="$(dirname "$0")/helpers:$PATH"
export PATH
exec /usr/bin/git "$@"
"#,
    );
    repository.git(["config", "protocol.owned.allow", "always"]);
    repository.git([
        "config",
        "remote.origin.url",
        "owned::https://name:URL_PASSWORD@example.invalid/repository",
    ]);
    helper
}

// An owned remote helper delegates to real Git credential fill, with a failing
// owned credential helper and no network. Actual noninteractive failure is
// classified without retaining its credential-bearing stderr or endpoint.
#[test]
fn failed_credentials() {
    for action in ["fetch", "push"] {
        let repository = Repository::new("sha1");
        ready(&repository);
        let helper = helper(
            &repository,
            r#"#!/bin/sh
printf '%s\n' "$GIT_TERMINAL_PROMPT" >> "$0.calls"
exec /usr/bin/git -c "credential.helper=$0.credential" credential fill <<'INPUT'
protocol=https
host=example.invalid

INPUT
"#,
        );
        executable(
            &helper.with_extension("credential"),
            r#"#!/bin/sh
printf 'CREDENTIAL_SENTINEL https://name:HELPER_PASSWORD@example.invalid\n' >&2
exit 1
"#,
        );
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let refs = observe(&mut owner);
        let id = if action == "fetch" {
            owner.fetch(refs, b"origin").unwrap()
        } else {
            let intent = prepare(&mut owner, refs);
            owner.push(intent).unwrap()
        };
        finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert_eq!(
            record.outcome,
            if action == "fetch" {
                Outcome::Failed
            } else {
                Outcome::Indeterminate
            },
            "{record:?}"
        );
        assert!(!record.exit.unwrap().success());
        assert!(
            record
                .diagnostic
                .as_ref()
                .unwrap()
                .contains("Noninteractive Git credential input is unavailable"),
            "{record:?}"
        );
        assert!(record.stdout.is_empty() && record.stderr.is_empty());
        let shown = format!("{record:?}");
        for secret in [
            "URL_PASSWORD",
            "HELPER_PASSWORD",
            "CREDENTIAL_SENTINEL",
            "https://name:",
        ] {
            assert!(!shown.contains(secret), "{action}");
        }
        assert!(
            fs::read_to_string(helper.with_extension("calls"))
                .unwrap()
                .lines()
                .all(|v| v == "0")
        );
        assert!(!owner.busy());
    }
}

// Quiet, excessive-output and cancelled real helpers remain owned under the
// unchanged process cleanup and five-second fixture bounds. Possible effects
// stay explicit and no diagnostic/credential data bypasses byte admission.
#[test]
fn bounded_transport() {
    for mode in ["deadline", "output", "cancel"] {
        let repository = Repository::new("sha1");
        ready(&repository);
        let body = if mode == "output" {
            r#"#!/bin/sh
: > "$0.started"
while :; do printf 'OUTPUT_CREDENTIAL_SENTINEL padding padding padding padding\n' >&2; done
"#
        } else {
            r#"#!/bin/sh
: > "$0.started"
sleep 30
"#
        };
        let helper = helper(&repository, body);
        let extra = if mode == "deadline" {
            "transport_timeout_seconds=2"
        } else if mode == "output" {
            "max_output_bytes=32768\nmax_detail_bytes=4096\nmax_diagnostic_bytes=1024"
        } else {
            ""
        };
        let mut owner = repository.owner(extra);
        repository.open(&mut owner, &repository.root);
        let refs = observe(&mut owner);
        let id = if mode == "cancel" {
            let intent = prepare(&mut owner, refs);
            owner.push(intent).unwrap()
        } else {
            owner.fetch(refs, b"origin").unwrap()
        };
        if mode == "cancel" {
            wait_path(&helper.with_extension("started"));
            owner.cancel(id).unwrap();
        }
        finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert_eq!(
            record.outcome,
            match mode {
                "deadline" => Outcome::TimedOut,
                "output" => Outcome::OutputLimit,
                _ => Outcome::Cancelled,
            },
            "{mode}: {record:?}"
        );
        assert_eq!(record.effects, Effects::Possible, "{mode}");
        assert!(helper.with_extension("started").exists(), "{mode}");
        assert!(record.stdout.is_empty() && record.stderr.is_empty());
        assert!(!format!("{record:?}").contains("CREDENTIAL_SENTINEL"));
        assert!(!owner.busy(), "{mode}");
        assert!(owner.admitted_bytes() <= rev::git::SESSION_BYTES);
    }
}
