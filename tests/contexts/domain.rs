use super::*;
use rev::buffer::Buffers;
use rev::coordinates::Scalar;
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::symlink;

// Names are display metadata, independent of both typed identity and filesystem
// naming restrictions. Case and Unicode remain meaningful within each type.
#[test]
fn names_and_identity() {
    for (label, accepted) in [
        ("  Δ / tree  ", true),
        ("Name", true),
        ("name", true),
        ("", false),
        (" \t ", false),
        ("bad\nname", false),
        (&"x".repeat(257), false),
    ] {
        assert_eq!(Name::new(label).is_ok(), accepted, "{label:?}");
    }
    assert_eq!(Name::new("  Δ / tree  ").unwrap().as_str(), "Δ / tree");
    assert_ne!(ProjectId::new().unwrap(), ProjectId::new().unwrap());
    assert!(ProjectId::try_from("directory-name".to_owned()).is_err());
    let mut catalog = Catalog::default();
    project(&mut catalog, Path::new("/example/a"), "Same");
    workspace(&mut catalog, "Same", vec![]);
    assert!(catalog.encode().is_ok());
    project(&mut catalog, Path::new("/example/b"), " Same ");
    assert!(matches!(catalog.encode(), Err(Error::Conflict(_))));
}

// Persisted columns count source scalars. Byte lengths, combining/wide glyphs,
// tab cells and CRLF must not move a restored point into another source line.
#[test]
fn source_positions() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a界\te\u{301}\r\nnext").unwrap();
    let buffer = buffers.get(id).unwrap();
    for (scalar, saved) in [
        (0, position(1, 1)),
        (2, position(1, 3)),
        (4, position(1, 5)),
        (7, position(2, 1)),
        (11, position(2, 5)),
    ] {
        assert_eq!(
            SourcePosition::capture(buffer, Scalar(scalar)).unwrap(),
            saved
        );
        assert_eq!(saved.restore(buffer).unwrap(), Scalar(scalar));
    }
    assert_eq!(position(1, 999).restore(buffer).unwrap(), Scalar(5));
    assert_eq!(position(99, 999).restore(buffer).unwrap(), Scalar(11));
    assert!(position(0, 1).restore(buffer).is_err());
    assert!(SourcePosition::capture(buffer, Scalar(99)).is_err());
}

// References select the most specific root after physical alias resolution;
// equivalent roots and duplicate canonical buffers never pass publication.
#[test]
fn aliases_and_overlap() {
    let fixture = fixture::Fixture::new();
    let file = fixture.write("outer/inner/source", "preserved");
    symlink(fixture.0.join("outer"), fixture.0.join("alias")).unwrap();
    let outer = root(&fixture.0.join("outer"));
    let inner = root(&fixture.0.join("outer/inner"));
    let roots = vec![outer.clone(), inner.clone()];
    let reference = FileReference::capture(&fixture.0.join("alias/inner/source"), &roots).unwrap();
    assert_eq!(
        reference,
        FileReference::RootRelative {
            root: inner.id,
            path: NativePath::relative(Path::new("source")).unwrap()
        }
    );
    assert_eq!(reference.resolve(&roots).unwrap(), file);
    for path in ["../escape", "a/../../escape", "/absolute", ""] {
        assert!(NativePath::relative(Path::new(path)).is_err(), "{path}");
    }
    let outside = fixture.write("outside", "outside");
    symlink(&outside, fixture.0.join("outer/escape")).unwrap();
    let escape = FileReference::RootRelative {
        root: outer.id.clone(),
        path: NativePath::relative(Path::new("escape")).unwrap(),
    };
    assert!(escape.resolve(&roots).is_err());
    symlink(&fixture.0, fixture.0.join("outer/outside-prefix")).unwrap();
    let missing_escape = FileReference::RootRelative {
        root: outer.id.clone(),
        path: NativePath::relative(Path::new("outside-prefix/not-created")).unwrap(),
    };
    assert!(missing_escape.resolve(&roots).is_err());
    assert!(matches!(
        FileReference::capture(&outside, &roots).unwrap(),
        FileReference::LooseAbsolute(_)
    ));
    let mut catalog = Catalog::default();
    project(&mut catalog, outer.path.as_path(), "A");
    project(&mut catalog, &fixture.0.join("alias"), "B");
    let prepared = Catalog::default().prepare(catalog).unwrap();
    let (store, _) = Store::open(&fixture.0.join("state/contexts.toml")).unwrap();
    assert!(matches!(store.publish(&prepared), Err(Error::Conflict(_))));
    assert_eq!(fs::read(file).unwrap(), b"preserved");
}

// Both wire forms are lossless and TOML escaping is owned by the existing
// encoder. Unavailable roots and stale provenance remain metadata, not sources.
#[test]
fn lossless_roundtrip() {
    let fixture = fixture::Fixture::new();
    let native = fixture.0.join(OsString::from_vec(b"native-\xff".to_vec()));
    fs::create_dir(&native).unwrap();
    let file = native.join(OsString::from_vec(b"file-\xfe".to_vec()));
    fs::write(&file, b"source").unwrap();
    let mut catalog = Catalog::default();
    project(&mut catalog, &native, "Quoted \"name\" \\ Δ");
    let reference = FileReference::capture(&file, &[catalog.projects[0].root.clone()]).unwrap();
    catalog.sessions[0].snapshot = Ok(Snapshot {
        files: vec![entry(reference.clone())],
        active_file: Some(reference),
    });
    let mut unavailable = root(&fixture.0.join("unavailable"));
    unavailable.origin_project_id = Some(ProjectId::new().unwrap());
    workspace(&mut catalog, "Empty then missing", vec![unavailable]);
    let bytes = catalog.encode().unwrap();
    assert!(std::str::from_utf8(&bytes).unwrap().contains("path_bytes"));
    assert_eq!(Catalog::decode(&bytes).unwrap(), catalog);
    assert_eq!(fs::read(file).unwrap(), b"source");
}

// Every revision edge is explicit. A rename preserves membership, while root
// changes require a matching session, and stale live preparation cannot activate.
#[test]
fn checked_revisions() {
    assert!(Revision::try_from(0).is_err());
    assert!(Revision::try_from(i64::MAX as u64).unwrap().next().is_err());
    let mut catalog = Catalog::default();
    project(&mut catalog, Path::new("/example/root"), "Old");
    let mut changed = catalog.clone();
    changed.projects[0].name = Name::new("New").unwrap();
    assert!(changed.clone().prepare(catalog.clone()).is_err());
    assert!(catalog.prepare(changed.clone()).is_err());
    changed.projects[0].revision = changed.projects[0].revision.next().unwrap();
    assert!(catalog.prepare(changed.clone()).is_ok());
    changed.projects[0].root.path = NativePath::absolute(Path::new("/example/moved")).unwrap();
    assert!(catalog.prepare(changed.clone()).is_err());
    changed.projects[0].roots_revision = Revision::INITIAL.next().unwrap();
    assert!(catalog.prepare(changed.clone()).is_err());
    changed.sessions[0].roots_revision = changed.projects[0].roots_revision;
    changed.sessions[0].reference.revision = Revision::INITIAL.next().unwrap();
    changed.projects[0].session_ref = changed.sessions[0].reference.clone();
    assert!(catalog.prepare(changed).is_ok());
    let prepared = PreparedContext {
        generation: Generation::INITIAL,
        destination: Context::Standalone,
    };
    assert!(
        prepared
            .activate(Generation::INITIAL.next().unwrap())
            .is_err()
    );
}

// Record references are validated together, so broken ownership, root IDs,
// active files and duplicate sessions cannot publish a partially valid catalog.
#[test]
fn reference_admission() {
    for case in [
        "owner",
        "orphan",
        "root",
        "active",
        "duplicate",
        "relative loose",
    ] {
        let mut catalog = Catalog::default();
        project(&mut catalog, Path::new("/example/root"), "Project");
        let reference = FileReference::RootRelative {
            root: catalog.projects[0].root.id.clone(),
            path: NativePath::relative(Path::new("file")).unwrap(),
        };
        let mut snapshot = Snapshot {
            files: vec![entry(reference.clone())],
            active_file: Some(reference.clone()),
        };
        match case {
            "owner" => {
                catalog.projects[0].session_ref.owner = Owner::Project(ProjectId::new().unwrap())
            }
            "orphan" => catalog.sessions.push(catalog.sessions[0].clone()),
            "root" => {
                snapshot.files[0].file = FileReference::RootRelative {
                    root: RootId::new().unwrap(),
                    path: NativePath::relative(Path::new("file")).unwrap(),
                }
            }
            "active" => snapshot.files.clear(),
            "duplicate" => snapshot.files.push(entry(reference)),
            "relative loose" => {
                snapshot.files[0].file =
                    FileReference::LooseAbsolute(NativePath::relative(Path::new("file")).unwrap())
            }
            _ => unreachable!(),
        }
        catalog.sessions[0].snapshot = Ok(snapshot);
        assert!(catalog.encode().is_err(), "{case}");
    }
}

// Native-byte and Unicode-name limits apply before expanded TOML encoding.
// Missing paths remain valid metadata and a full root list remains admissible.
#[test]
fn native_bounds() {
    assert!(
        NativePath::absolute(Path::new(&format!("/{}", "x".repeat(MAX_PATH_BYTES - 1)))).is_ok()
    );
    assert!(NativePath::absolute(Path::new(&format!("/{}", "x".repeat(MAX_PATH_BYTES)))).is_err());
    assert!(NativePath::absolute(Path::new("/nul\0path")).is_err());
    assert!(Name::new(&"界".repeat(85)).is_ok());
    assert!(Name::new(&"界".repeat(86)).is_err());
    let mut catalog = Catalog::default();
    let roots = (0..MAX_ROOTS)
        .map(|i| root(&Path::new("/example").join(format!("root-{i}"))))
        .collect();
    workspace(&mut catalog, "Full roots", roots);
    assert_eq!(
        Catalog::decode(&catalog.encode().unwrap()).unwrap(),
        catalog
    );
}

// Overlapping tree routes are valid, but a saved session cannot publish two
// references for the same physical file through distinct root identities.
#[test]
fn canonical_files() {
    let fixture = fixture::Fixture::new();
    let source = fixture.write("outer/inner/source", "kept");
    let outer = root(&fixture.0.join("outer"));
    let inner = root(&fixture.0.join("outer/inner"));
    let mut catalog = Catalog::default();
    workspace(
        &mut catalog,
        "Overlapping",
        vec![outer.clone(), inner.clone()],
    );
    let reference = FileReference::capture(&source, &[outer.clone(), inner]).unwrap();
    catalog.sessions[0].snapshot = Ok(Snapshot {
        files: vec![
            entry(reference.clone()),
            entry(FileReference::RootRelative {
                root: outer.id,
                path: NativePath::relative(Path::new("inner/source")).unwrap(),
            }),
        ],
        active_file: Some(reference.clone()),
    });
    let (store, empty) = Store::open(&fixture.0.join("state/contexts.toml")).unwrap();
    assert!(matches!(
        store.publish(&empty.prepare(catalog.clone()).unwrap()),
        Err(Error::Conflict(_))
    ));
    catalog.sessions[0].snapshot.as_mut().unwrap().files.pop();
    let published = store.publish(&empty.prepare(catalog).unwrap()).unwrap();
    assert_eq!(
        published.catalog.sessions[0]
            .snapshot
            .as_ref()
            .unwrap()
            .files[0]
            .file,
        reference
    );
}
