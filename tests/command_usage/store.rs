use crate::fixture::Fixture;
use rev::command::usage::{Error, Store};
use rev::file::Output;
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

pub(super) fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).into()).collect()
}
pub(super) fn disk(path: &Path) -> Vec<String> {
    Store::open(path).unwrap().1
}

// Opening and no-op publication are passive. New state, lock and parent
// directories appear only after an accepted event and retain user-only modes.
#[test]
fn missing_state() {
    let fixture = Fixture::new();
    let path = fixture.0.join("state/rev/command-history.toml");
    let (mut store, loaded) = Store::open(&path).unwrap();
    assert!(loaded.is_empty());
    assert!(store.publish(&[]).unwrap().is_empty());
    assert!(!path.parent().unwrap().exists());
    assert_eq!(
        store.publish(&names(&["find-file"])).unwrap(),
        ["find-file"]
    );
    assert_eq!(disk(&path), ["find-file"]);
    for (target, mode) in [
        (path.clone(), 0o600),
        (path.with_extension("toml.lock"), 0o600),
        (path.parent().unwrap().into(), 0o700),
    ] {
        assert_eq!(
            fs::metadata(target).unwrap().permissions().mode() & 0o777,
            mode
        );
    }
}

// Strict versioned records reject unknown fields, malformed names/types and
// excessive events. Removed but structurally valid names remain historical data.
#[test]
fn format_admission() {
    let fixture = Fixture::new();
    let many = vec!["undo"; 257];
    let long = "a".repeat(129);
    let cases = [
        ("empty", "version=1\ncommands=[]".into(), true),
        (
            "repeated",
            "version=1\ncommands=['undo','undo']".into(),
            true,
        ),
        ("removed", "version=1\ncommands=['ai-removed']".into(), true),
        (
            "max name",
            format!("version=1\ncommands=['{}']", "a".repeat(128)),
            true,
        ),
        ("version", "version=2\ncommands=[]".into(), false),
        ("missing", "version=1".into(), false),
        (
            "unknown",
            "version=1\ncommands=[]\nextra=true".into(),
            false,
        ),
        ("type", "version=1\ncommands='undo'".into(), false),
        (
            "count",
            format!(
                "version=1\ncommands={}",
                serde_json::to_string(&many).unwrap()
            ),
            false,
        ),
        (
            "name bytes",
            format!("version=1\ncommands=['{long}']"),
            false,
        ),
        ("uppercase", "version=1\ncommands=['Undo']".into(), false),
        ("digit", "version=1\ncommands=['1undo']".into(), false),
        ("space", "version=1\ncommands=['undo now']".into(), false),
        ("unicode", "version=1\ncommands=['undo-界']".into(), false),
        ("empty name", "version=1\ncommands=['']".into(), false),
        ("malformed", "private-invalid-content".into(), false),
    ];
    for (case, text, accepted) in cases {
        let path = fixture.write("history.toml", &text);
        let result = Store::open(&path);
        assert_eq!(result.is_ok(), accepted, "{case}");
        assert_eq!(fs::read(&path).unwrap(), text.as_bytes(), "{case}");
        assert!(!path.with_extension("toml.lock").exists());
    }
}

// Reads reject oversized/nonordinary/unreadable inputs before parsing or
// waiting on them. Exact-size valid input and maximum output remain bounded.
#[test]
fn storage_bounds() {
    let fixture = Fixture::new();
    let valid = "version=1\ncommands=[]\n";
    for length in [65536, 65537] {
        let mut bytes = valid.as_bytes().to_vec();
        bytes.resize(length, b' ');
        let path = fixture.0.join("history.toml");
        fs::write(&path, &bytes).unwrap();
        assert_eq!(Store::open(&path).is_ok(), length == 65536);
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    let directory = fixture.0.join("directory");
    fs::create_dir(&directory).unwrap();
    assert!(Store::open(&directory).is_err());
    let link = fixture.0.join("link");
    symlink("missing", &link).unwrap();
    assert!(Store::open(&link).is_err());
    assert_eq!(fs::read_link(link).unwrap(), Path::new("missing"));
    let path = fixture.write("read-only.toml", valid);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    assert!(Store::open(&path).is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let (mut store, _) = Store::open(&path).unwrap();
    let batch = vec!["a".repeat(128); 256];
    assert_eq!(store.publish(&batch).unwrap(), batch);
    assert!(fs::metadata(&path).unwrap().len() <= 65536);
    let before = fs::read(&path).unwrap();
    for invalid in [
        vec!["undo".into(); 257],
        vec!["a".repeat(129)],
        names(&["bad name"]),
    ] {
        assert_eq!(store.publish(&invalid).unwrap_err(), Error::Bound);
        assert_eq!(fs::read(&path).unwrap(), before);
    }
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(store.publish(&names(&["newline"])).is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
}

// Publishers loaded from the same startup snapshot rebase onto current disk,
// including removed names, and retain only the newest 256 complete events.
#[test]
fn publisher_rebase() {
    let fixture = Fixture::new();
    let path = fixture.0.join("history.toml");
    let (mut first, _) = Store::open(&path).unwrap();
    let (mut second, _) = Store::open(&path).unwrap();
    first.publish(&vec!["undo".into(); 255]).unwrap();
    second.publish(&names(&["ai-removed", "newline"])).unwrap();
    let window = disk(&path);
    assert_eq!(window.len(), 256);
    assert_eq!(&window[254..], ["ai-removed", "newline"]);
    assert_eq!(window.iter().filter(|name| *name == "undo").count(), 254);
}

// Real simultaneous lock owners either publish or defer. A later bounded retry
// retains each successfully published batch exactly once, in publication order.
#[test]
fn concurrent_publishers() {
    let fixture = Fixture::new();
    let path = fixture.0.join("history.toml");
    let barrier = Arc::new(Barrier::new(2));
    // Eager array mapping starts both barrier participants before either join.
    let tasks = ["undo", "newline"].map(|name| {
        let barrier = Arc::clone(&barrier);
        let path = path.clone();
        std::thread::spawn(move || {
            let (mut store, _) = Store::open(&path).unwrap();
            barrier.wait();
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                match store.publish(&names(&[name])) {
                    Ok(window) => return window,
                    Err(Error::Busy) if Instant::now() < deadline => std::thread::yield_now(),
                    Err(error) => panic!("{error}"),
                }
            }
        })
    });
    let windows: Vec<_> = tasks.into_iter().map(|task| task.join().unwrap()).collect();
    let final_window = disk(&path);
    assert!(final_window == ["undo", "newline"] || final_window == ["newline", "undo"]);
    assert!(windows.iter().any(|window| window == &final_window));
}

// Busy and substituted lock paths preserve existing state. No publication waits
// for a cooperating holder, and releasing that holder permits an explicit retry.
#[test]
fn lock_admission() {
    let fixture = Fixture::new();
    let path = fixture.write("history.toml", "version=1\ncommands=['undo']");
    let lock = path.with_extension("toml.lock");
    let holder = File::create(&lock).unwrap();
    holder.try_lock().unwrap();
    let (mut store, _) = Store::open(&path).unwrap();
    assert_eq!(
        store.publish(&names(&["newline"])).unwrap_err(),
        Error::Busy
    );
    assert_eq!(disk(&path), ["undo"]);
    holder.unlock().unwrap();
    assert_eq!(
        store.publish(&names(&["newline"])).unwrap(),
        ["undo", "newline"]
    );
    drop(holder);
    fs::remove_file(&lock).unwrap();
    symlink(&path, &lock).unwrap();
    assert!(store.publish(&names(&["save-buffer"])).is_err());
    assert_eq!(disk(&path), ["undo", "newline"]);
}

struct Fault(&'static str);
impl Output for Fault {
    fn flush(&mut self, file: &mut File) -> io::Result<()> {
        if self.0 == "flush" {
            return Err(io::Error::other("fixture failure"));
        }
        file.flush()
    }
    fn sync_file(&mut self, file: &File) -> io::Result<()> {
        if self.0 == "file" {
            return Err(io::Error::other("fixture failure"));
        }
        file.sync_all()
    }
    fn replace(&mut self, source: &Path, target: &Path) -> io::Result<()> {
        if self.0 == "replace" {
            return Err(io::Error::other("fixture failure"));
        }
        fs::rename(source, target)
    }
    fn sync_directory(&mut self, directory: &File) -> io::Result<()> {
        if self.0 == "directory" {
            return Err(io::Error::other("fixture failure"));
        }
        directory.sync_all()
    }
    fn observe(&mut self, path: &Path) -> io::Result<rev::file::Observation> {
        if self.0 == "observation" {
            return Err(io::Error::other("fixture failure"));
        }
        rev::file::observe(path)?.ok_or_else(|| io::Error::other("missing output"))
    }
}

// Real staging faults retain old bytes and allow a safe retry. Post-replacement
// faults leave a complete record and suspend retries that would duplicate counts.
#[test]
fn publication_failures() {
    for point in ["flush", "file", "replace", "directory", "observation"] {
        let fixture = Fixture::new();
        let path = fixture.write("history.toml", "version=1\ncommands=['undo']");
        let before = fs::read(&path).unwrap();
        let (mut store, _) = Store::open(&path).unwrap();
        let failed = store.publish_with(&names(&["newline"]), &mut Fault(point));
        assert!(failed.is_err(), "{point}");
        let published = matches!(point, "directory" | "observation");
        if published {
            assert_eq!(disk(&path), ["undo", "newline"]);
            assert_eq!(
                store.publish(&names(&["newline"])).unwrap_err(),
                Error::Ambiguous
            );
            assert_eq!(disk(&path), ["undo", "newline"]);
        } else {
            assert_eq!(fs::read(&path).unwrap(), before, "{point}");
            assert_eq!(
                store.publish(&names(&["newline"])).unwrap(),
                ["undo", "newline"]
            );
        }
        assert!(fs::read_dir(&fixture.0).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".rev-save-")
        }));
    }
}
