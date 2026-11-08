use super::*;
use crate::file::{Files, Output};
use crate::fixture::Fixture;
use std::fs::{self, File};
use std::path::Path;

struct Fault(&'static str);
impl Output for Fault {
    fn flush(&mut self, file: &mut File) -> std::io::Result<()> {
        if self.0 == "flush" {
            Err(std::io::Error::other("flush fault"))
        } else {
            Files.flush(file)
        }
    }
    fn sync_file(&mut self, file: &File) -> std::io::Result<()> {
        if self.0 == "file" {
            Err(std::io::Error::other("sync fault"))
        } else {
            Files.sync_file(file)
        }
    }
    fn replace(&mut self, temporary: &Path, target: &Path) -> std::io::Result<()> {
        if self.0 == "replace" {
            Err(std::io::Error::other("replace fault"))
        } else {
            Files.replace(temporary, target)
        }
    }
    fn sync_directory(&mut self, file: &File) -> std::io::Result<()> {
        if self.0 == "directory" {
            Err(std::io::Error::other("directory fault"))
        } else {
            Files.sync_directory(file)
        }
    }
    fn observe(&mut self, path: &Path) -> std::io::Result<crate::file::Observation> {
        if self.0 == "observe" {
            Err(std::io::Error::other("observe fault"))
        } else {
            Files.observe(path)
        }
    }
}

// Native replacement is the commit point. Before it, old bytes and drafts
// survive; afterward the exact candidate identity/revision survives uncertainty.
#[test]
fn publication_faults() {
    for stage in ["flush", "file", "replace", "directory", "observe"] {
        let fixture = Fixture::new();
        let roots = crate::config::Roots {
            config: fixture.0.join("config"),
            state: fixture.0.join("state"),
            data: fixture.0.join("data"),
            cache: fixture.0.join("cache"),
        };
        let store = Store::at_state(&roots).unwrap();
        let first = super::codec::tests::record("alpha");
        store.save(first.clone(), None).unwrap();
        let path = roots.state.join("macros/library.toml");
        let before = fs::read(&path).unwrap();
        let mut candidate = first.clone();
        candidate.revision = 2;
        candidate.description = Some("changed".into());
        let result = store.publish(
            Some(candidate.clone()),
            "alpha",
            Some((&first.id, 1)),
            &mut Fault(stage),
        );
        if ["directory", "observe"].contains(&stage) {
            assert!(result.unwrap().warning);
            let disk = store.load().unwrap();
            assert!(disk.macros[0] == candidate);
            // Retrying that exact candidate cannot add a record or revision.
            assert!(matches!(
                store.publish(
                    Some(candidate.clone()),
                    "alpha",
                    Some((&first.id, 1)),
                    &mut Files
                ),
                Err(Error::Conflict(_))
            ));
            assert!(store.load().unwrap().macros[0] == candidate);
        } else {
            assert!(result.is_err());
            assert_eq!(fs::read(&path).unwrap(), before);
        }
        assert_eq!(fs::read_dir(roots.state.join("macros")).unwrap().count(), 2);
        let other = super::codec::tests::record("bravo");
        assert!(store.save(other, None).is_ok(), "lock released at {stage}");
    }
}

// Checked exhaustion cannot wrap a saved identity or aggregate revision, evict
// a record, or damage valid bytes. Releasing the lock remains unconditional.
#[test]
fn revision_exhaustion() {
    for record_exhausted in [false, true] {
        let fixture = Fixture::new();
        let roots = crate::config::Roots {
            config: fixture.0.join("config"),
            state: fixture.0.join("state"),
            data: fixture.0.join("data"),
            cache: fixture.0.join("cache"),
        };
        let store = Store::at_state(&roots).unwrap();
        let mut record = super::codec::tests::record("alpha");
        store.save(record.clone(), None).unwrap();
        let mut catalog = store.load().unwrap();
        if record_exhausted {
            catalog.macros[0].revision = i64::MAX as u64;
            record.revision = i64::MAX as u64;
        } else {
            catalog.revision = i64::MAX as u64;
        }
        let path = roots.state.join("macros/library.toml");
        let bytes = catalog.encode().unwrap();
        fs::write(&path, &bytes).unwrap();
        let saved = Saved {
            id: record.id.clone(),
            revision: record.revision,
            baseline: std::sync::Arc::new(record.definition(1).unwrap()),
            description: None,
            binding: None,
            active_binding: None,
        };
        assert!(matches!(
            store.save(record, Some(&saved)),
            Err(Error::Capacity(_))
        ));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(roots.state.join("macros/library.toml.lock"))
            .unwrap();
        lock.try_lock().unwrap();
    }
}
