use super::*;
use std::path::{Path, PathBuf};

fn policy(f: &Fixture, idle: usize) -> Pi {
    Pi {
        command: support::peer(f, "normal").to_str().unwrap().into(),
        idle_timeout_ms: idle,
        request_timeout_ms: 1000,
        ..Pi::default()
    }
}
fn root(f: &Fixture) -> PathBuf {
    let p = f.0.join("project");
    fs::create_dir(&p).unwrap();
    p
}
fn generate(f: &Fixture, root: &Path, policy: Pi, reset: bool) -> Operation {
    Operation::retained(
        policy,
        root.to_owned(),
        r#"{"instruction":"fixture only","dirty_sources":[]}"#.into(),
        101,
        Some(f.0.join("state")),
        reset,
    )
    .unwrap()
}
fn index(f: &Fixture) -> toml::Value {
    toml::from_str(&fs::read_to_string(f.0.join("state/ai/pi/sessions.toml")).unwrap()).unwrap()
}
fn session(f: &Fixture) -> PathBuf {
    f.0.join("state/ai/pi/sessions")
        .join(index(f)["bindings"][0]["session"].as_str().unwrap())
}
fn close(mut op: Operation) {
    op.cancel();
    support::until(|| op.worker_finished());
    let _ = op.take();
    assert!(!op.cleanup_failed());
}

// Idle reuse retains exactly this process, refreshes invocation state, then
// disconnect/restart restores only its private model/root/grant binding.
#[test]
fn retained_history() {
    let f = Fixture::new();
    let root = root(&f);
    let mut policy = policy(&f, 5000);
    if let Some(executable) = std::env::var_os("REV_PI_EXECUTABLE") {
        use std::os::unix::fs::PermissionsExt;
        let actual = std::path::PathBuf::from(executable).canonicalize().unwrap();
        let extension =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ai/discovery-provider.js");
        let wrapper = f.0.join("actual-pi");
        fs::write(&wrapper,format!("#!/usr/bin/python3\nimport os,sys\nos.execv({exe},[{exe},*sys.argv[1:],'--extension',{extension}])\n",exe=serde_json::to_string(actual.to_str().unwrap()).unwrap(),extension=serde_json::to_string(extension.to_str().unwrap()).unwrap())).unwrap();
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700)).unwrap();
        policy.command = wrapper.to_str().unwrap().into();
        policy.model = Some("rev-fixture/local".into());
        policy.request_timeout_ms = 5000;
        f.write("project/main.zig", "disk only");
        f.write("project/fixture-tools-per-invocation", "16");
    }
    let mut op = generate(&f, &root, policy.clone(), false);
    assert!(finish(&mut op).unwrap().history.contains("new"));
    assert!(op.can_reuse());
    assert!(!op.worker_finished());
    let pid = if std::env::var_os("REV_PI_EXECUTABLE").is_none() {
        Some(fs::read_to_string(f.0.join("pid")).unwrap())
    } else {
        Some(fs::read_to_string(root.join("fixture-pid")).unwrap())
    };
    let path = session(&f);
    op.resume(r#"{"instruction":"second","dirty_sources":[]}"#.into(), 102)
        .unwrap();
    assert!(finish(&mut op).unwrap().history.contains("resumed"));
    if let Some(pid) = &pid {
        let current = if std::env::var_os("REV_PI_EXECUTABLE").is_some() {
            fs::read_to_string(root.join("fixture-pid")).unwrap()
        } else {
            fs::read_to_string(f.0.join("pid")).unwrap()
        };
        assert_eq!(current, *pid);
    }
    assert!(fs::read_to_string(&path).unwrap().contains("second"));
    close(op);
    if let Some(pid) = &pid {
        assert!(!Path::new("/proc").join(pid).exists());
    }
    let mut restored = generate(&f, &root, policy, false);
    assert!(finish(&mut restored).unwrap().history.contains("resumed"));
    assert_eq!(session(&f), path);
    assert!(fs::metadata(&path).unwrap().len() > 0);
    assert_eq!(index(&f)["version"].as_integer(), Some(1));
    if std::env::var_os("REV_PI_EXECUTABLE").is_none() {
        let limits: Vec<u64> =
            serde_json::from_str(&fs::read_to_string(f.0.join("file-limit")).unwrap()).unwrap();
        assert_eq!(limits, [33554432, 33554432]);
    }
    if std::env::var_os("REV_PI_EXECUTABLE").is_some() {
        let limit = fs::read_to_string(root.join("fixture-limit")).unwrap();
        assert!(limit.contains("33554432"), "{limit}");
        let context: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join("fixture-context")).unwrap())
                .unwrap();
        assert_eq!(context["has_second"], true);
        assert!(context["users"].as_u64().unwrap() >= 3);
    }
    close(restored);
}

// Different roots, resolved providers/models and exact grants isolate history.
// Explicit reset removes this owned history only and assigns a fresh Pi path.
#[test]
fn binding_boundaries() {
    let f = Fixture::new();
    let root = root(&f);
    let policy = policy(&f, 0);
    let mut first = generate(&f, &root, policy.clone(), false);
    finish(&mut first).unwrap();
    let original = session(&f);
    let mut changed = policy.clone();
    changed.model = Some("another/model".into());
    finish(&mut generate(&f, &root, changed, false)).unwrap();
    let mut narrowed = policy.clone();
    narrowed.max_input_bytes /= 2;
    finish(&mut generate(&f, &root, narrowed, false)).unwrap();
    let other = f.0.join("other");
    fs::create_dir(&other).unwrap();
    finish(&mut generate(&f, &other, policy.clone(), false)).unwrap();
    assert_eq!(index(&f)["bindings"].as_array().unwrap().len(), 4);
    f.write("resolved-model", "default-changed/other");
    let mut observed = generate(&f, &root, policy.clone(), false);
    assert_eq!(
        finish(&mut observed).unwrap().model,
        "default-changed/other"
    );
    f.write("resolved-model", "fixture/controlled");
    assert_eq!(index(&f)["bindings"].as_array().unwrap().len(), 5);
    let mut reset = generate(&f, &root, policy.clone(), true);
    assert!(finish(&mut reset).unwrap().history.contains("new"));
    assert!(reset.reset_applied());
    assert!(!original.exists());
    assert_ne!(session(&f), original);
    assert_eq!(index(&f)["bindings"].as_array().unwrap().len(), 5);
    let retained = session(&f);
    fs::rename(&root, f.0.join("retired-root")).unwrap();
    fs::create_dir(&root).unwrap();
    assert!(
        finish(&mut generate(&f, &root, policy, false))
            .unwrap()
            .history
            .contains("new")
    );
    assert_eq!(index(&f)["bindings"].as_array().unwrap().len(), 6);
    assert!(retained.exists());
}

// A live history lease blocks another editor without losing index updates.
// The idle deadline exits the owned process and releases that lease naturally.
#[test]
fn concurrent_and_idle() {
    let f = Fixture::new();
    let root = root(&f);
    let policy = policy(&f, 1000);
    let mut live = generate(&f, &root, policy.clone(), false);
    finish(&mut live).unwrap();
    let mut rival = generate(&f, &root, policy.clone(), false);
    assert!(finish(&mut rival).unwrap_err().contains("another editor"));
    let other = f.0.join("another-root");
    fs::create_dir(&other).unwrap();
    let mut independent = generate(&f, &other, policy.clone(), false);
    finish(&mut independent).unwrap();
    assert_eq!(index(&f)["bindings"].as_array().unwrap().len(), 2);
    close(independent);
    support::until(|| live.worker_finished());
    let mut again = generate(&f, &root, policy, false);
    finish(&mut again).unwrap();
    close(again);
}

// Invalid metadata and unsafe/over-limit files are preserved, never guessed as
// another last session or silently truncated. Explicit ephemeral work still runs.
#[test]
fn invalid_history() {
    for case in ["schema", "corrupt", "link", "mode", "size", "index-size"] {
        let f = Fixture::new();
        let root = root(&f);
        let policy = policy(&f, 0);
        finish(&mut generate(&f, &root, policy.clone(), false)).unwrap();
        let path = session(&f);
        let index_path = f.0.join("state/ai/pi/sessions.toml");
        match case {
            "schema" => {
                let text = fs::read_to_string(&index_path).unwrap().replacen(
                    "version = 1",
                    "version = 9",
                    1,
                );
                fs::write(&index_path, text).unwrap();
            }
            "corrupt" => fs::write(&index_path, "invalid [").unwrap(),
            "index-size" => fs::write(&index_path, "x".repeat(65537)).unwrap(),
            "link" => {
                fs::remove_file(&path).unwrap();
                std::os::unix::fs::symlink(f.0.join("outside"), &path).unwrap();
            }
            "mode" => {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
            }
            "size" => std::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .unwrap()
                .set_len(33554432)
                .unwrap(),
            _ => unreachable!(),
        }
        let before = fs::read(&index_path).unwrap();
        assert!(
            finish(&mut generate(&f, &root, policy.clone(), false)).is_err(),
            "{case}"
        );
        assert_eq!(fs::read(&index_path).unwrap(), before, "{case}");
        let mut ephemeral = policy;
        ephemeral.persist_sessions = false;
        assert!(
            finish(&mut generate(&f, &root, ephemeral, false)).is_ok(),
            "{case}"
        );
        assert_eq!(fs::read(&index_path).unwrap(), before, "{case}");
    }
}

// Capacity is explicit and non-evicting; a rejected seventeenth binding cannot
// mutate metadata or another history. Reset at capacity reuses its owned slot.
#[test]
fn binding_capacity() {
    let f = Fixture::new();
    let root = root(&f);
    let base = policy(&f, 0);
    for n in 0..16 {
        let mut selected = base.clone();
        selected.model = Some(format!("fixture/model-{n}"));
        finish(&mut generate(&f, &root, selected, false)).unwrap();
    }
    let before = fs::read(f.0.join("state/ai/pi/sessions.toml")).unwrap();
    let mut extra = base.clone();
    extra.model = Some("fixture/seventeenth".into());
    assert!(
        finish(&mut generate(&f, &root, extra, false))
            .unwrap_err()
            .contains("16")
    );
    assert_eq!(
        fs::read(f.0.join("state/ai/pi/sessions.toml")).unwrap(),
        before
    );
    let mut reset = base;
    reset.model = Some("fixture/model-0".into());
    finish(&mut generate(&f, &root, reset, true)).unwrap();
    assert_eq!(index(&f)["bindings"].as_array().unwrap().len(), 16);
}

// Compaction notifications preserve invocation accounting. Editing the private
// index while generation is pending rejects completion without overwriting it.
#[test]
fn compaction_and_conflicts() {
    for mode in ["compaction", "compaction_turns", "settle_gate"] {
        let f = Fixture::new();
        let root = root(&f);
        let policy = policy(&f, 0);
        f.write("mode", mode);
        let mut op = generate(&f, &root, policy, false);
        if mode == "settle_gate" {
            support::until(|| f.0.join("partial").exists());
            let path = f.0.join("state/ai/pi/sessions.toml");
            fs::write(&path, "invalid [").unwrap();
            f.write("release", "");
            assert!(finish(&mut op).is_err());
            assert_eq!(fs::read_to_string(path).unwrap(), "invalid [");
        } else if mode == "compaction_turns" {
            assert!(finish(&mut op).unwrap_err().contains("eight"));
        } else {
            assert!(finish(&mut op).is_ok());
        }
    }
}
