use super::*;
use rev::file::{Observation, Output};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::sync::{Arc, Barrier};

fn seeded(fixture: &fixture::Fixture) -> (Store, Catalog, std::path::PathBuf) {
    let path = fixture.0.join("state/contexts.toml");
    let (store, empty) = Store::open(&path).unwrap();
    let mut catalog = empty.clone();
    let source = fixture.write("source/file", "unchanged source");
    project(&mut catalog, source.parent().unwrap(), "Project");
    let reference = FileReference::capture(&source, &[catalog.projects[0].root.clone()]).unwrap();
    catalog.sessions[0].snapshot = Ok(Snapshot {
        files: vec![entry(reference.clone())],
        active_file: Some(reference),
    });
    let published = store.publish(&empty.prepare(catalog).unwrap()).unwrap();
    assert!(published.durability_warning.is_none());
    (store, published.catalog, path)
}
fn renamed(catalog: &Catalog, name: &str) -> Prepared {
    let mut next = catalog.clone();
    next.projects[0].name = Name::new(name).unwrap();
    next.projects[0].revision = next.projects[0].revision.next().unwrap();
    catalog.prepare(next).unwrap()
}
fn converted(catalog: &Catalog) -> Prepared {
    let mut next = catalog.clone();
    let project = next.projects.remove(0);
    let old = next.sessions.remove(0);
    let mut new_root = project.root;
    new_root.id = RootId::new().unwrap();
    new_root.origin_project_id = Some(project.id);
    workspace(&mut next, "Converted", vec![new_root.clone()]);
    let remap = |reference: &FileReference| match reference {
        FileReference::RootRelative { path, .. } => FileReference::RootRelative {
            root: new_root.id.clone(),
            path: path.clone(),
        },
        other => other.clone(),
    };
    let old = old.snapshot.unwrap();
    next.sessions[0].snapshot = Ok(Snapshot {
        files: old
            .files
            .into_iter()
            .map(|entry| FileEntry {
                file: remap(&entry.file),
                ..entry
            })
            .collect(),
        active_file: old.active_file.as_ref().map(remap),
    });
    catalog.prepare(next).unwrap()
}

// Read-only catalog access and canceled preparation create no state. First
// explicit publication creates only private owned metadata, never source files.
#[test]
fn passive_reads() {
    let fixture = fixture::Fixture::new();
    let path = fixture.0.join("state/rev/contexts.toml");
    let (store, empty) = Store::open(&path).unwrap();
    assert_eq!(empty, Catalog::default());
    let mut next = empty.clone();
    workspace(&mut next, "Empty", vec![]);
    drop(empty.prepare(next.clone()).unwrap());
    assert!(!path.parent().unwrap().exists());
    store.publish(&empty.prepare(next).unwrap()).unwrap();
    for (target, mode) in [
        (path.clone(), 0o600),
        (path.with_extension("toml.lock"), 0o600),
        (path.parent().unwrap().to_owned(), 0o700),
    ] {
        assert_eq!(
            fs::metadata(target).unwrap().permissions().mode() & 0o777,
            mode
        );
    }
    let loaded = Store::open(&path).unwrap().1;
    assert_eq!(loaded.workspaces.len(), 1);
    assert!(loaded.workspaces[0].roots.is_empty());
}

// Global expected state rejects both stale metadata and same-owner sessions.
// One nonblocking lock attempt never waits or claims a competing write succeeded.
#[test]
fn stale_and_busy_writers() {
    let fixture = fixture::Fixture::new();
    let (store, base, path) = seeded(&fixture);
    let (other, captured) = Store::open(&path).unwrap();
    let first = renamed(&base, "First");
    let stale = renamed(&captured, "Stale");
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.with_extension("toml.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    assert!(matches!(store.publish(&first), Err(Error::Busy)));
    lock.unlock().unwrap();
    let winner = store.publish(&first).unwrap().catalog;
    assert!(matches!(other.publish(&stale), Err(Error::Conflict(_))));
    assert!(matches!(store.publish(&first), Err(Error::Conflict(_))));
    assert_eq!(Store::open(&path).unwrap().1, winner);
    let mut session_update = winner.clone();
    session_update.projects[0].revision = session_update.projects[0].revision.next().unwrap();
    session_update.projects[0].session_ref.revision = session_update.projects[0]
        .session_ref
        .revision
        .next()
        .unwrap();
    session_update.sessions[0].reference = session_update.projects[0].session_ref.clone();
    session_update.sessions[0].snapshot.as_mut().unwrap().files[0].cursor = position(9, 9);
    let prepared = winner.prepare(session_update).unwrap();
    let newer = store.publish(&prepared).unwrap().catalog;
    assert!(other.publish(&prepared).is_err());
    assert_eq!(Store::open(&path).unwrap().1, newer);
}

// Two actual threads race independent candidates from the same revision. Only
// one wins; the other reports busy/conflict instead of silently dropping state.
#[test]
fn concurrent_publication() {
    let fixture = fixture::Fixture::new();
    let (_, base, path) = seeded(&fixture);
    let barrier = Arc::new(Barrier::new(2));
    // Eager array mapping starts both barrier participants before either join.
    let threads = ["One", "Two"].map(|name| {
        let path = path.clone();
        let base = base.clone();
        let barrier = Arc::clone(&barrier);
        std::thread::spawn(move || {
            let (store, _) = Store::open(&path).unwrap();
            let prepared = renamed(&base, name);
            barrier.wait();
            store.publish(&prepared)
        })
    });
    let outcomes: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    assert!(
        outcomes
            .iter()
            .any(|result| matches!(result, Err(Error::Busy | Error::Conflict(_))))
    );
    assert!(matches!(
        Store::open(&path).unwrap().1.projects[0].name.as_str(),
        "One" | "Two"
    ));
}

struct Fault(&'static str);
impl Output for Fault {
    fn flush(&mut self, file: &mut File) -> io::Result<()> {
        if self.0 == "flush" {
            Err(io::Error::other("injected flush failure"))
        } else {
            std::io::Write::flush(file)
        }
    }
    fn sync_file(&mut self, file: &File) -> io::Result<()> {
        if self.0 == "file sync" {
            Err(io::Error::other("injected sync failure"))
        } else {
            file.sync_all()
        }
    }
    fn replace(&mut self, temporary: &Path, target: &Path) -> io::Result<()> {
        if self.0 == "replace" {
            Err(io::Error::other("injected replacement failure"))
        } else {
            fs::rename(temporary, target)
        }
    }
    fn sync_directory(&mut self, directory: &File) -> io::Result<()> {
        if self.0 == "directory sync" {
            Err(io::Error::other("injected durability failure"))
        } else {
            directory.sync_all()
        }
    }
    fn observe(&mut self, path: &Path) -> io::Result<Observation> {
        if self.0 == "observe" {
            Err(io::Error::other("injected observation failure"))
        } else {
            rev::file::observe(path)?.ok_or_else(|| io::Error::other("missing published file"))
        }
    }
}

// Real aggregate replacements exercise both sides of the conversion commit
// point. Restart sees either the complete Project or complete remapped Workspace,
// with source bytes intact and no abandoned staging files.
#[test]
fn atomic_conversion() {
    for phase in [
        "flush",
        "file sync",
        "replace",
        "directory sync",
        "observe",
        "success",
    ] {
        let fixture = fixture::Fixture::new();
        let (store, before, path) = seeded(&fixture);
        let bytes = fs::read(&path).unwrap();
        let prepared = converted(&before);
        let published = store.publish_with(&prepared, &mut Fault(phase));
        let after_commit = matches!(phase, "directory sync" | "observe" | "success");
        assert_eq!(published.is_ok(), after_commit, "{phase}");
        let loaded = Store::open(&path).unwrap().1;
        if after_commit {
            assert!(loaded.projects.is_empty(), "{phase}");
            assert_eq!(loaded.workspaces.len(), 1);
            assert_eq!(loaded, *prepared.candidate());
            assert_eq!(
                published.unwrap().durability_warning.is_some(),
                phase != "success"
            );
            let root = &loaded.workspaces[0].roots[0];
            let snapshot = loaded.sessions[0].snapshot.as_ref().unwrap();
            assert!(
                matches!(&snapshot.files[0].file, FileReference::RootRelative { root: id, .. } if *id == root.id)
            );
            assert_eq!(snapshot.active_file.as_ref(), Some(&snapshot.files[0].file));
        } else {
            assert_eq!(loaded, before, "{phase}");
            assert_eq!(fs::read(&path).unwrap(), bytes, "{phase}");
        }
        assert_eq!(
            fs::read(fixture.0.join("source/file")).unwrap(),
            b"unchanged source"
        );
        assert_eq!(
            fs::read_dir(path.parent().unwrap()).unwrap().count(),
            2,
            "{phase}"
        );
    }
}

// Corruption and unsupported/oversized inputs remain byte-for-byte intact when
// read or encountered by a previously prepared writer; no empty reset is allowed.
#[test]
fn damaged_input_preserved() {
    for case in ["malformed", "unsupported", "oversized", "unknown field"] {
        let fixture = fixture::Fixture::new();
        let (store, base, path) = seeded(&fixture);
        let prepared = renamed(&base, "Must not publish");
        let bytes = match case {
            "malformed" => b"invalid = [ private broken input".to_vec(),
            "unsupported" => String::from_utf8(base.encode().unwrap())
                .unwrap()
                .replacen("schema_version = 1", "schema_version = 2", 1)
                .into_bytes(),
            "oversized" => vec![b' '; MAX_CATALOG_BYTES + 1],
            _ => b"schema_version=1\nrevision=1\nunknown=true\n".to_vec(),
        };
        fs::write(&path, &bytes).unwrap();
        assert!(Store::open(&path).is_err(), "{case}");
        assert!(store.publish(&prepared).is_err(), "{case}");
        assert_eq!(fs::read(&path).unwrap(), bytes, "{case}");
    }
}

// A damaged session is scoped to its typed owner and retained across unrelated
// changes, while catalog identity and source bytes remain available for recovery.
#[test]
fn damaged_session_isolated() {
    let fixture = fixture::Fixture::new();
    let (store, base, path) = seeded(&fixture);
    let bytes = String::from_utf8(base.encode().unwrap()).unwrap();
    let broken = bytes.replace("line = 2", "line = 0");
    fs::write(&path, &broken).unwrap();
    let loaded = Store::open(&path).unwrap().1;
    assert!(loaded.sessions[0].snapshot.is_err());
    assert_eq!(loaded.projects[0].id, base.projects[0].id);
    let candidate = renamed(&loaded, "Renamed with retained session");
    let published = store.publish(&candidate).unwrap();
    let reloaded = Store::open(&path).unwrap().1;
    assert_eq!(reloaded, published.catalog);
    assert!(reloaded.sessions[0].snapshot.is_err());
    assert!(
        String::from_utf8(fs::read(&path).unwrap())
            .unwrap()
            .contains("line = 0")
    );
}

// Final and intermediate symlink substitutions, FIFO/device inputs and locks
// never grant write authority to another directory or metadata target.
#[test]
fn storage_substitution() {
    for case in ["catalog", "lock", "parent", "ancestor", "directory", "fifo"] {
        let fixture = fixture::Fixture::new();
        let (store, base, path) = seeded(&fixture);
        let target = fixture.write("unrelated/keep", "unrelated bytes");
        let prepared = renamed(&base, "Rejected");
        match case {
            "catalog" => {
                fs::remove_file(&path).unwrap();
                symlink(&target, &path).unwrap();
            }
            "lock" => {
                let lock = path.with_extension("toml.lock");
                fs::remove_file(&lock).unwrap();
                symlink(&target, lock).unwrap();
            }
            "parent" | "ancestor" => {
                fs::rename(path.parent().unwrap(), fixture.0.join("retained")).unwrap();
                symlink(fixture.0.join("unrelated"), path.parent().unwrap()).unwrap();
            }
            "directory" => {
                fs::remove_file(&path).unwrap();
                fs::create_dir(&path).unwrap();
            }
            "fifo" => {
                fs::remove_file(&path).unwrap();
                rustix::fs::mkfifoat(
                    rustix::fs::CWD,
                    &path,
                    rustix::fs::Mode::from_raw_mode(0o600),
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
        assert!(store.publish(&prepared).is_err(), "{case}");
        assert_eq!(fs::read(target).unwrap(), b"unrelated bytes", "{case}");
    }
}

// Counts and encoded bytes are admitted together. Rejection never truncates the
// selected durable snapshot. Maximum file count and empty Workspaces are valid.
#[test]
fn capacity_admission() {
    let fixture = fixture::Fixture::new();
    let (store, base, path) = seeded(&fixture);
    let bytes = fs::read(&path).unwrap();
    for case in [
        "projects",
        "workspaces",
        "roots",
        "files",
        "snapshot bytes",
        "catalog bytes",
    ] {
        let mut next = base.clone();
        match case {
            "projects" => {
                for i in 1..=MAX_PROJECTS {
                    project(
                        &mut next,
                        &fixture.0.join(format!("p{i}")),
                        &format!("P{i}"),
                    );
                }
            }
            "workspaces" => {
                for i in 0..=MAX_WORKSPACES {
                    workspace(&mut next, &format!("W{i}"), vec![]);
                }
            }
            "roots" => {
                let roots = (0..=MAX_ROOTS)
                    .map(|i| root(&fixture.0.join(format!("r{i}"))))
                    .collect();
                workspace(&mut next, "Many", roots);
            }
            "files" | "snapshot bytes" => {
                let width = if case == "snapshot bytes" { 4000 } else { 8 };
                let count = if case == "snapshot bytes" {
                    MAX_FILES
                } else {
                    MAX_FILES + 1
                };
                let files = (0..count)
                    .map(|i| {
                        entry(FileReference::LooseAbsolute(
                            NativePath::absolute(
                                &Path::new("/").join(format!("{i}{}", "\\".repeat(width))),
                            )
                            .unwrap(),
                        ))
                    })
                    .collect();
                next.sessions[0].snapshot = Ok(Snapshot {
                    files,
                    active_file: None,
                });
                next.sessions[0].reference.revision = Revision::INITIAL.next().unwrap();
                next.projects[0].session_ref = next.sessions[0].reference.clone();
                next.projects[0].revision = next.projects[0].revision.next().unwrap();
            }
            "catalog bytes" => {
                for i in 0..8 {
                    workspace(&mut next, &format!("W{i}"), vec![]);
                    let files = (0..MAX_FILES)
                        .map(|j| {
                            entry(FileReference::LooseAbsolute(
                                NativePath::absolute(
                                    &Path::new("/").join(format!("{j}{}", "a".repeat(2200))),
                                )
                                .unwrap(),
                            ))
                        })
                        .collect();
                    next.sessions.last_mut().unwrap().snapshot = Ok(Snapshot {
                        files,
                        active_file: None,
                    });
                }
            }
            _ => unreachable!(),
        }
        assert!(base.prepare(next).is_err(), "{case}");
        assert_eq!(fs::read(&path).unwrap(), bytes, "{case}");
    }
    let mut next = base.clone();
    workspace(&mut next, "Max files", vec![]);
    next.sessions.last_mut().unwrap().snapshot = Ok(Snapshot {
        files: (0..MAX_FILES)
            .map(|i| {
                entry(FileReference::LooseAbsolute(
                    NativePath::absolute(&fixture.0.join(format!("loose-{i}"))).unwrap(),
                ))
            })
            .collect(),
        active_file: None,
    });
    assert!(store.publish(&base.prepare(next).unwrap()).is_ok());
}

// Substitution after staging is checked before the replacement point. Preserve
// a changed catalog and both renamed directories instead of following a new
// pathname or recreating an unlocked lockfile.
#[test]
fn staging_substitution() {
    struct Change {
        phase: &'static str,
        path: std::path::PathBuf,
    }
    impl Output for Change {
        fn flush(&mut self, file: &mut File) -> io::Result<()> {
            std::io::Write::flush(file)?;
            match self.phase {
                "catalog" => fs::write(&self.path, b"changed externally"),
                "lock" => {
                    let lock = self.path.with_extension("toml.lock");
                    fs::rename(&lock, self.path.with_extension("retained-lock"))?;
                    fs::write(lock, b"replacement lock")
                }
                "parent" => {
                    let parent = self.path.parent().unwrap();
                    fs::rename(parent, parent.with_extension("retained"))?;
                    fs::create_dir(parent)
                }
                _ => unreachable!(),
            }
        }
    }
    for phase in ["catalog", "lock", "parent"] {
        let fixture = fixture::Fixture::new();
        let (store, base, path) = seeded(&fixture);
        let before = fs::read(&path).unwrap();
        assert!(
            store
                .publish_with(
                    &renamed(&base, "Rejected"),
                    &mut Change {
                        phase,
                        path: path.clone()
                    }
                )
                .is_err(),
            "{phase}"
        );
        match phase {
            "catalog" => assert_eq!(fs::read(&path).unwrap(), b"changed externally"),
            "lock" => assert_eq!(fs::read(&path).unwrap(), before),
            "parent" => {
                assert!(!path.exists());
                assert_eq!(
                    fs::read(
                        path.parent()
                            .unwrap()
                            .with_extension("retained")
                            .join("contexts.toml")
                    )
                    .unwrap(),
                    before
                );
            }
            _ => unreachable!(),
        }
        assert_eq!(
            fs::read(fixture.0.join("source/file")).unwrap(),
            b"unchanged source"
        );
    }
}

// A source root observed during preparation cannot be replaced before commit.
// Retained unavailable roots still survive a later unrelated catalog mutation.
#[test]
fn root_observations() {
    let fixture = fixture::Fixture::new();
    let (store, base, path) = seeded(&fixture);
    let bytes = fs::read(&path).unwrap();
    let prepared = renamed(&base, "Rejected replacement");
    fs::rename(fixture.0.join("source"), fixture.0.join("retained-source")).unwrap();
    fs::create_dir(fixture.0.join("source")).unwrap();
    assert!(matches!(store.publish(&prepared), Err(Error::Conflict(_))));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::remove_dir(fixture.0.join("source")).unwrap();
    let prepared = renamed(&base, "Retained unavailable root");
    assert!(store.publish(&prepared).is_ok());
    let current = Store::open(&path).unwrap().1;
    assert_eq!(current.projects[0].root.id, base.projects[0].root.id);
    assert_eq!(
        fs::read(fixture.0.join("retained-source/file")).unwrap(),
        b"unchanged source"
    );
    let mut next = current.clone();
    project(
        &mut next,
        &fixture.0.join("never-created"),
        "Unavailable new root",
    );
    assert!(store.publish(&current.prepare(next).unwrap()).is_err());
    assert_eq!(Store::open(&path).unwrap().1, current);
}
