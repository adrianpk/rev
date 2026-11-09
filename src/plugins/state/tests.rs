use super::*;
use crate::{
    file::{Files, Output},
    fixture::Fixture,
};
use std::{
    fs::{self, File},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

struct Fault<'a> {
    stage: &'a str,
    live: Arc<AtomicBool>,
}
impl Output for Fault<'_> {
    fn flush(&mut self, file: &mut File) -> std::io::Result<()> {
        if self.stage == "flush" {
            return Err(std::io::Error::other("flush fault"));
        }
        Files.flush(file)
    }
    fn sync_file(&mut self, file: &File) -> std::io::Result<()> {
        if self.stage == "file" {
            return Err(std::io::Error::other("sync fault"));
        }
        if self.stage == "cancel" {
            self.live.store(false, Ordering::Release);
        }
        Files.sync_file(file)
    }
    fn replace(&mut self, temporary: &Path, target: &Path) -> std::io::Result<()> {
        if self.stage == "replace" {
            Err(std::io::Error::other("replace fault"))
        } else {
            Files.replace(temporary, target)
        }
    }
    fn sync_directory(&mut self, file: &File) -> std::io::Result<()> {
        if self.stage == "directory" {
            Err(std::io::Error::other("directory fault"))
        } else {
            Files.sync_directory(file)
        }
    }
    fn observe(&mut self, path: &Path) -> std::io::Result<crate::file::Observation> {
        if self.stage == "observe" {
            Err(std::io::Error::other("observe fault"))
        } else {
            Files.observe(path)
        }
    }
}
fn put(expected: Option<&str>, content: &str) -> Operation {
    Operation::Put(StatePutParams {
        concern: RecordId::new("notes").unwrap(),
        record: RecordId::new("alpha").unwrap(),
        expected_version: ExpectedVersion(expected.map(|v| Revision::new(v).unwrap())),
        value: serde_json::json!(content),
    })
}
fn get() -> Operation {
    Operation::Get(StateGetParams {
        concern: RecordId::new("notes").unwrap(),
        record: RecordId::new("alpha").unwrap(),
    })
}

// Real native replacement is the commit point. Only deterministic IO fault
// injection is substituted; reads, writes, locking and replacement remain real.
#[test]
fn publication_faults() {
    for stage in ["flush", "file", "replace", "cancel", "directory", "observe"] {
        let fixture = Fixture::new();
        let package = PackageId::new("field.notes").unwrap();
        let mut store = Store::new(&fixture.0, &package);
        store.execute(put(None, "first"), || true).unwrap();
        let before = fs::read(&store.path).unwrap();
        let live = Arc::new(AtomicBool::new(true));
        let result = store.execute_with(
            put(Some("1"), "candidate"),
            || live.load(Ordering::Acquire),
            &mut Fault {
                stage,
                live: Arc::clone(&live),
            },
        );
        assert!(result.is_err(), "{stage}");
        if ["directory", "observe"].contains(&stage) {
            let error = result.unwrap_err();
            assert_eq!(error.kind(), Some(ErrorKind::StateConflict));
            assert!(error.message.contains("committed at version 2"));
            assert_ne!(fs::read(&store.path).unwrap(), before);
            assert!(store.execute(put(Some("2"), "blocked"), || true).is_err());
            let current = store.execute(get(), || true).unwrap();
            assert_eq!(current["value"], "candidate");
            assert_eq!(current["version"], "2");
            assert!(
                store
                    .execute(put(Some("1"), "stale retry"), || true)
                    .is_err()
            );
            store
                .execute(put(Some("2"), "reconciled"), || true)
                .unwrap();
        } else {
            assert_eq!(fs::read(&store.path).unwrap(), before);
            store.execute(put(Some("1"), "retry"), || true).unwrap();
        }
        assert_eq!(
            fs::read_dir(store.path.parent().unwrap()).unwrap().count(),
            2,
            "owned staging cleaned at {stage}"
        );
    }
}

// Monotonic aggregate versions survive record deletion, preventing ABA after
// recreation. Exhaustion refuses publication without damaging existing bytes.
#[test]
fn version_exhaustion() {
    let fixture = Fixture::new();
    let mut store = Store::new(&fixture.0, &PackageId::new("field.notes").unwrap());
    store.execute(put(None, "first"), || true).unwrap();
    let bytes = format!(
        "schema_version=1\nrevision={}\n[[records]]\nconcern='notes'\nrecord='alpha'\nversion={}\nvalue_json='null'\n",
        i64::MAX,
        i64::MAX
    );
    fs::write(&store.path, &bytes).unwrap();
    let result = store
        .execute(put(Some(&i64::MAX.to_string()), "overflow"), || true)
        .unwrap_err();
    assert_eq!(result.kind(), Some(ErrorKind::LimitExceeded));
    assert_eq!(fs::read_to_string(&store.path).unwrap(), bytes);
}

struct Substitute {
    package: std::path::PathBuf,
    stage: &'static str,
    protected: std::path::PathBuf,
}
impl Output for Substitute {
    fn sync_file(&mut self, file: &File) -> std::io::Result<()> {
        Files.sync_file(file)?;
        match self.stage {
            "parent" => self.substitute_parent()?,
            "lock" => self.substitute_lock()?,
            "temporary" => self.substitute_temporary()?,
            _ => unreachable!(),
        }
        Ok(())
    }
}
impl Substitute {
    fn substitute_parent(&self) -> std::io::Result<()> {
        fs::rename(&self.package, self.package.with_file_name("retained"))?;
        fs::create_dir(&self.package)?;
        fs::write(self.package.join("unrelated"), b"preserved")?;
        Ok(())
    }
    fn substitute_lock(&self) -> std::io::Result<()> {
        fs::rename(
            self.package.join("state.toml.lock"),
            self.package.join("retained-lock"),
        )?;
        std::os::unix::fs::symlink(&self.protected, self.package.join("state.toml.lock"))?;
        Ok(())
    }
    fn substitute_temporary(&self) -> std::io::Result<()> {
        let path = fs::read_dir(&self.package)?
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".rev-save-")
            })
            .unwrap();
        fs::remove_file(&path)?;
        std::os::unix::fs::symlink(&self.protected, path)?;
        Ok(())
    }
}
// Deterministic substitutions at the native IO boundary cannot redirect a
// commit or turn a staging pathname into permission to delete an unknown entry.
#[test]
fn substituted_publication() {
    for stage in ["parent", "lock", "temporary"] {
        let fixture = Fixture::new();
        let mut store = Store::new(&fixture.0, &PackageId::new("field.notes").unwrap());
        store.execute(put(None, "first"), || true).unwrap();
        let before = fs::read(&store.path).unwrap();
        let protected = fixture.0.join("protected");
        fs::write(&protected, b"unrelated").unwrap();
        let package = store.path.parent().unwrap().to_owned();
        assert!(
            store
                .execute_with(
                    put(Some("1"), "candidate"),
                    || true,
                    &mut Substitute {
                        package: package.clone(),
                        stage,
                        protected: protected.clone()
                    }
                )
                .is_err()
        );
        assert_eq!(fs::read(&protected).unwrap(), b"unrelated");
        if stage == "parent" {
            assert_eq!(
                fs::read(package.with_file_name("retained").join("state.toml")).unwrap(),
                before
            );
            assert_eq!(fs::read(package.join("unrelated")).unwrap(), b"preserved");
        } else {
            assert_eq!(fs::read(&store.path).unwrap(), before);
            if stage == "temporary" {
                assert!(
                    fs::read_dir(package).unwrap().any(|e| e
                        .unwrap()
                        .file_type()
                        .unwrap()
                        .is_symlink())
                );
            }
        }
    }
}
