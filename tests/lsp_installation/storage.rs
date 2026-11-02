use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::time::{Duration, Instant};

use rev::buffer::Buffers;
use rev::config::{Roots, SettingsEdit, Snapshot};
use rev::lsp::install::{Launch, Manager, Registry, State, Storage};

use crate::fixture::Fixture;

pub(super) fn roots(f: &Fixture) -> Roots {
    Roots {
        config: f.0.join("config"),
        data: f.0.join("data"),
        cache: f.0.join("cache"),
        state: f.0.join("state"),
    }
}

pub(super) fn launch() -> Launch {
    Launch {
        id: "rust-analyzer".into(),
        languages: vec!["rust".into()],
        executable: "rust-analyzer".into(),
        args: Vec::new(),
        env: Default::default(),
        root_markers: vec!["Cargo.toml".into()],
        initialization_options: toml::Value::Table(Default::default()),
        settings: toml::Value::Table(Default::default()),
    }
}

// Preparing/ready facts round trip independently of settings. Ready records
// require owned executable paths and integrity/probe timestamps, not presence.
#[test]
fn registry_admission() {
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let mut candidate = storage
        .candidate(
            "rust-analyzer",
            "2026-11-02",
            "x86_64-unknown-linux-gnu",
            "https://example.test/release",
        )
        .unwrap();
    assert!(candidate.directory.starts_with(&roots.data));
    assert_eq!(
        Registry::load(&roots.data).unwrap().installations[0].status,
        State::Preparing
    );
    let executable = candidate.directory.join("rust-analyzer");
    fs::write(&executable, "owned executable").unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    candidate.installation.launches.push(launch());
    candidate.installation.status = State::Ready;
    assert!(
        storage
            .save(candidate.installation.clone())
            .unwrap_err()
            .contains("integrity")
    );
    candidate.installation.evidence = format!("sha256:{}", "a".repeat(64));
    assert!(
        storage
            .save(candidate.installation.clone())
            .unwrap_err()
            .contains("timestamp")
    );
    candidate.installation.verified_at = Some(candidate.installation.created_at);
    assert!(
        storage
            .save(candidate.installation.clone())
            .unwrap()
            .is_none()
    );
    assert_eq!(
        Registry::load(&roots.data).unwrap().installations,
        vec![candidate.installation.clone()]
    );
    assert!(!roots.config.exists());
    let registry = roots.data.join("lsp/installations.toml");
    let valid = fs::read_to_string(&registry).unwrap();
    drop(storage);
    let cases = [
        ("schema", valid.replacen("schema = 1", "schema = 2", 1)),
        (
            "absolute",
            valid.replace(
                &format!("payload = \"{}\"", candidate.installation.payload.display()),
                "payload = '/escape'",
            ),
        ),
        (
            "traversal",
            valid.replace("executable = \"rust-analyzer\"", "executable = '../escape'"),
        ),
        (
            "unverified",
            valid.replace(&candidate.installation.evidence, "unverified"),
        ),
        (
            "duplicate",
            format!("{valid}\n{}", valid.strip_prefix("schema = 1\n").unwrap()),
        ),
        ("malformed", "schema = 'private-value'".into()),
    ];
    for (name, invalid) in cases {
        fs::write(&registry, &invalid).unwrap();
        assert!(Registry::load(&roots.data).is_err(), "{name}");
        assert!(Storage::acquire(roots.clone()).is_err(), "{name}");
        assert_eq!(fs::read_to_string(&registry).unwrap(), invalid, "{name}");
    }
    fs::write(&registry, &valid).unwrap();
    fs::remove_file(&executable).unwrap();
    let outside = f.write("external", "external executable");
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o700)).unwrap();
    symlink(&outside, &executable).unwrap();
    assert!(Registry::load(&roots.data).unwrap_err().contains("escapes"));
    fs::write(&registry, vec![b' '; rev::lsp::install::REGISTRY_BYTES + 1]).unwrap();
    assert!(Registry::load(&roots.data).unwrap_err().contains("2 MiB"));
}

// A data-root lock spans processes even with distinct state roots. Settings
// publication shares the state lock and cannot race a live management job.
#[test]
fn mutation_ownership() {
    if let Some(data) = std::env::var_os("REV_INSTALL_LOCK_DATA") {
        let f = Fixture::new();
        let mut roots = roots(&f);
        roots.data = data.into();
        assert!(Storage::acquire(roots).err().unwrap().contains("data root"));
        return;
    }
    let f = Fixture::new();
    let roots = roots(&f);
    let storage = Storage::acquire(roots.clone()).unwrap();
    let root = f.write("config/settings.toml", "# unchanged\n");
    let edit = SettingsEdit::capture(&root).unwrap();
    assert!(
        edit.publish(&roots.state)
            .err()
            .unwrap()
            .to_string()
            .contains("mutation lock")
    );
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "storage::mutation_ownership", "--nocapture"])
        .env("REV_INSTALL_LOCK_DATA", &roots.data)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    drop(storage);
    assert!(Storage::acquire(roots).is_ok());
}

// A manual registry replacement is preserved. Cancellation does not release a
// worker's slot early, and idle polling never creates roots or launches work.
#[test]
fn worker_and_conflicts() {
    let f = Fixture::new();
    let roots = roots(&f);
    let mut manager = Manager::default();
    assert!(manager.poll().is_none());
    assert!(!roots.data.exists());
    let buffers = Buffers::with_settings(Snapshot::default()).unwrap();
    let stamp = buffers.get(buffers.initial()).unwrap().stamp();
    manager
        .start(stamp, roots.clone(), |mut storage, cancel| {
            let candidate = storage
                .candidate("rust-analyzer", "test", "fixture", "https://example.test")
                .unwrap();
            while !cancel.is_cancelled() {
                std::thread::sleep(Duration::from_millis(2));
            }
            let mut incomplete = candidate.installation;
            incomplete.status = State::Incomplete;
            storage.save(incomplete)?;
            Err("cancelled owned work".into())
        })
        .unwrap();
    assert!(manager.active());
    assert!(
        manager
            .start(stamp, roots.clone(), |_, _| unreachable!())
            .is_err()
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while !roots.data.join("lsp/installations.toml").exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(manager.cancel());
    loop {
        assert!(Instant::now() < deadline);
        if let Some(result) = manager.poll() {
            assert!(result.err().unwrap().contains("cancelled"));
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!manager.active());
    assert_eq!(manager.phase(), rev::lsp::install::Phase::Cancelled);
    assert_eq!(
        Registry::load(&roots.data).unwrap().installations[0].status,
        State::Incomplete
    );
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let installation = storage.registry().installations[0].clone();
    let path = roots.data.join("lsp/installations.toml");
    fs::write(&path, "not valid").unwrap();
    assert!(storage.save(installation).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), "not valid");
}

// Capacity and payload bounds apply before publication. An escaping recipe
// directory is rejected before creating any path through that link.
#[test]
fn storage_limits() {
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let mut candidate = storage
        .candidate("rust-analyzer", "test", "fixture", "https://example.test")
        .unwrap();
    let executable = candidate.directory.join("rust-analyzer");
    fs::write(&executable, "owned").unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    candidate.installation.launches.push(launch());
    candidate.installation.status = State::Ready;
    candidate.installation.evidence = format!("sha256:{}", "a".repeat(64));
    candidate.installation.verified_at = Some(candidate.installation.created_at);
    fs::File::options()
        .write(true)
        .open(&executable)
        .unwrap()
        .set_len(rev::lsp::install::PAYLOAD_BYTES + 1)
        .unwrap();
    assert!(
        storage
            .save(candidate.installation)
            .unwrap_err()
            .contains("bytes")
    );
    assert_eq!(
        Registry::load(&roots.data).unwrap().installations[0].status,
        State::Preparing
    );
    let outside = f.0.join("outside");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, roots.data.join("lsp/installations/escape")).unwrap();
    assert!(
        storage
            .candidate("escape", "test", "fixture", "https://example.test")
            .err()
            .unwrap()
            .contains("escapes")
    );
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    for _ in 1..rev::lsp::install::RECORDS {
        storage
            .candidate("rust-analyzer", "test", "fixture", "https://example.test")
            .unwrap();
    }
    assert!(
        storage
            .candidate("rust-analyzer", "test", "fixture", "https://example.test")
            .is_err()
    );
    assert_eq!(
        Registry::load(&roots.data).unwrap().installations.len(),
        rev::lsp::install::RECORDS
    );
}
