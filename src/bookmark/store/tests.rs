use super::*;
use crate::bookmark::{Context, MAX_ENTRIES};
use crate::buffer::Buffers;
use crate::coordinates::Scalar;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rev-bookmark-store-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> PathBuf {
        self.0.join("data/bookmarks.toml")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn collection(name: &str) -> Bookmarks {
    let buffers = Buffers::new().unwrap();
    let buffer = buffers.get(buffers.initial()).unwrap();
    let mut records = Bookmarks::default();
    records
        .set(
            name,
            PathBuf::from("/intended/missing"),
            crate::buffer::edit::Point {
                stamp: buffer.stamp(),
                position: Scalar(0),
            },
            true,
            Context::capture(buffer, Scalar(0)).unwrap(),
        )
        .unwrap();
    records
}

// Strict candidate rejection is atomic: no partial list escapes, and reads
// leave unknown versions, malformed evidence and oversized stores untouched.
#[test]
fn strict_admission() {
    let f = Fixture::new();
    fs::create_dir(f.0.join("data")).unwrap();
    let valid = String::from_utf8(format::encode(&collection("A")).unwrap()).unwrap();
    let variants = [
        (valid.replace("version = 1", "version = 2"), "version"),
        (format!("{valid}unknown = true\n"), "unknown"),
        (
            format!(
                "{valid}{}",
                valid.split_once("\n[[bookmarks]]").unwrap().1.replacen(
                    "\n",
                    "\n[[bookmarks]]\n",
                    1
                )
            ),
            "duplicate",
        ),
        (valid.replace("position = 0", "position = -1"), "negative"),
        (
            valid.replace("position = 0", "position = 9223372036854775808"),
            "overflow",
        ),
        (
            valid.replace("at_start = true", "at_start = false"),
            "context",
        ),
        (
            valid.replace("path = \"/intended/missing\"", "path = \"relative\""),
            "relative",
        ),
        (
            valid.replace(
                "path = \"/intended/missing\"",
                "path = \"/ok\"\npath_bytes = [47, 97]",
            ),
            "both paths",
        ),
        (
            valid.replace("name = \"A\"", &format!("name = \"{}\"", "n".repeat(257))),
            "name cap",
        ),
        (
            valid.replace("/intended/missing", &format!("/{}", "p".repeat(4096))),
            "path cap",
        ),
    ];
    for (bytes, label) in variants {
        fs::write(f.path(), &bytes).unwrap();
        assert!(Store::open(&f.path(), MAX_ENTRIES).is_err(), "{label}");
        assert_eq!(fs::read(f.path()).unwrap(), bytes.as_bytes(), "{label}");
    }
    fs::write(f.path(), valid).unwrap();
    assert!(Store::open(&f.path(), 0).is_err());
    let file = OpenOptions::new().write(true).open(f.path()).unwrap();
    file.set_len(LIMIT as u64 + 1).unwrap();
    assert!(
        Store::open(&f.path(), MAX_ENTRIES)
            .err()
            .unwrap()
            .contains("16 MiB")
    );
    assert_eq!(file.metadata().unwrap().len(), LIMIT as u64 + 1);
}

// Native non-UTF-8 paths and context escaping round trip without lossy text;
// reloading gets fresh IDs and no live buffer/revision association.
#[test]
fn lossless_round_trip() {
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let mut records = collection("界\"\\");
    let record = records.entries.values_mut().next().unwrap();
    record.path = PathBuf::from(std::ffi::OsString::from_vec(b"/native/\xff".to_vec()));
    record.context.after = "\n\t\0界".into();
    let (mut store, loaded) = Store::open(&f.path(), MAX_ENTRIES).unwrap();
    assert!(loaded.is_empty());
    store.write(&records).unwrap();
    let encoded = fs::read_to_string(f.path()).unwrap();
    assert!(encoded.contains("path_bytes = ["));
    assert!(!encoded.contains("generation"));
    assert_eq!(
        fs::metadata(f.path()).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(f.path().parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    let (_, loaded) = Store::open(&f.path(), MAX_ENTRIES).unwrap();
    let previous = records.iter().next().unwrap().id();
    let path = records.iter().next().unwrap().path().to_owned();
    records.admit(loaded).unwrap();
    let reloaded = records.iter().next().unwrap();
    assert_eq!(reloaded.path(), path);
    assert_ne!(reloaded.id(), previous);
    assert_eq!(
        reloaded.point().stamp.buffer,
        crate::buffer::BufferId::UNBOUND
    );
    assert_eq!(reloaded.resolution(), Resolution::Unresolved);
    assert!(!records.is_dirty());
}

struct Fault(&'static str);
impl Output for Fault {
    fn flush(&mut self, file: &mut File) -> std::io::Result<()> {
        if self.0 == "flush" {
            Err(std::io::Error::other("staged fault"))
        } else {
            file.flush()
        }
    }
    fn sync_file(&mut self, file: &File) -> std::io::Result<()> {
        if self.0 == "sync" {
            Err(std::io::Error::other("staged fault"))
        } else {
            file.sync_all()
        }
    }
    fn replace(&mut self, temporary: &Path, target: &Path) -> std::io::Result<()> {
        if self.0 == "replace" {
            Err(std::io::Error::other("replace fault"))
        } else {
            fs::rename(temporary, target)
        }
    }
    fn sync_directory(&mut self, dir: &File) -> std::io::Result<()> {
        if self.0 == "directory" {
            Err(std::io::Error::other("directory fault"))
        } else {
            dir.sync_all()
        }
    }
    fn observe(&mut self, path: &Path) -> std::io::Result<Observation> {
        if self.0 == "observe" {
            Err(std::io::Error::other("observation fault"))
        } else {
            Files.observe(path)
        }
    }
}

// Faults before rename preserve old bytes. Faults after rename retain explicit
// uncertainty; retry reconciles actual candidate bytes, never a metadata guess.
#[test]
fn publication_faults() {
    for fault in ["flush", "sync", "replace", "directory", "observe"] {
        let f = Fixture::new();
        let (mut store, _) = Store::open(&f.path(), MAX_ENTRIES).unwrap();
        store.write(&collection("old")).unwrap();
        let old = fs::read(f.path()).unwrap();
        let changed = collection("new");
        let error = store.write_with(&changed, &mut Fault(fault)).unwrap_err();
        assert!(changed.is_dirty());
        if matches!(fault, "directory" | "observe") {
            assert!(error.contains("completion unresolved"));
            assert_eq!(
                fs::read(f.path()).unwrap(),
                format::encode(&changed).unwrap()
            );
            assert!(store.pending.is_some());
        } else {
            assert_eq!(fs::read(f.path()).unwrap(), old);
            assert!(store.pending.is_none());
        }
        assert!(
            !fs::read_dir(f.path().parent().unwrap())
                .unwrap()
                .any(|entry| entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".rev-save-"))
        );
        store.write(&changed).unwrap();
        assert!(store.pending.is_none());
        assert_eq!(
            fs::read(f.path()).unwrap(),
            format::encode(&changed).unwrap()
        );
    }
    let f = Fixture::new();
    let (mut store, _) = Store::open(&f.path(), MAX_ENTRIES).unwrap();
    assert!(
        store
            .write_with(&collection("pending"), &mut Fault("observe"))
            .is_err()
    );
    let external = format::encode(&collection("external")).unwrap();
    fs::write(f.path(), &external).unwrap();
    assert!(store.write(&collection("retry")).is_err());
    assert_eq!(fs::read(f.path()).unwrap(), external);
}

fn wait(path: &Path) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(8);
    while !path.exists() {
        assert!(
            std::time::Instant::now() < end,
            "subprocess handshake timed out: {path:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
fn child(dir: &Path, role: &str) -> std::process::Child {
    std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "bookmark::store::tests::process_writer",
            "--nocapture",
        ])
        .env("REV_BOOKMARK_STORE_CHILD", dir)
        .env("REV_BOOKMARK_STORE_ROLE", role)
        .spawn()
        .unwrap()
}

// Two real processes first load the same absence. A writes; B's stale observed
// absence must not overwrite it. A separate held OS lock is immediately busy.
#[test]
fn process_conflict() {
    let f = Fixture::new();
    let mut a = child(&f.0, "A");
    let mut b = child(&f.0, "B");
    wait(&f.0.join("ready-A"));
    wait(&f.0.join("ready-B"));
    fs::write(f.0.join("go-A"), "").unwrap();
    wait(&f.0.join("done-A"));
    fs::write(f.0.join("go-B"), "").unwrap();
    assert!(a.wait().unwrap().success());
    assert!(b.wait().unwrap().success());
    let bytes = fs::read(f.path()).unwrap();
    assert_eq!(bytes, format::encode(&collection("A")).unwrap());
    let mut lock = child(&f.0, "lock");
    wait(&f.0.join("ready-lock"));
    let (mut store, _) = Store::open(&f.path(), MAX_ENTRIES).unwrap();
    assert!(
        store
            .write(&collection("blocked"))
            .unwrap_err()
            .contains("busy")
    );
    assert_eq!(fs::read(f.path()).unwrap(), bytes);
    fs::write(f.0.join("go-lock"), "").unwrap();
    assert!(lock.wait().unwrap().success());
    store.write(&collection("after-lock")).unwrap();
}

// Child entrypoint is inert in the normal unit run; each invocation owns only
// its loaded observation and the parent's isolated handshake directory.
#[test]
fn process_writer() {
    let Some(dir) = std::env::var_os("REV_BOOKMARK_STORE_CHILD") else {
        return;
    };
    let dir = PathBuf::from(dir);
    let path = dir.join("data/bookmarks.toml");
    let role = std::env::var("REV_BOOKMARK_STORE_ROLE").unwrap();
    let (mut store, _) = Store::open(&path, MAX_ENTRIES).unwrap();
    let lock = if role == "lock" {
        Some(store.lock().unwrap())
    } else {
        None
    };
    fs::write(dir.join(format!("ready-{role}")), "").unwrap();
    wait(&dir.join(format!("go-{role}")));
    if role != "lock" {
        let records = collection(&role);
        let result = store.write(&records);
        if role == "A" {
            result.unwrap();
        } else {
            assert!(result.unwrap_err().contains("changed externally"));
            assert!(records.is_dirty());
        }
        fs::write(dir.join(format!("done-{role}")), "").unwrap();
    }
    drop(lock);
}
