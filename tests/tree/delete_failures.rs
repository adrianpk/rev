use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::{PermissionsExt, symlink};

use rev::buffer::{Buffers, SaveRequest};
use rev::file::{DeleteDecision, Deletion};
use rev::tree::Tree;

use super::super::rename::{insert, text};
use super::super::{Fixture, choose};
use super::confirmed;

fn recursive(path: &std::path::Path) -> Deletion {
    let DeleteDecision::Recursive(deletion) = Deletion::capture(path).unwrap().confirm().unwrap()
    else {
        panic!("missing recursive decision");
    };
    confirmed(deletion)
}

// Links to files, directories and missing targets remove only their own entry.
// Recursive deletion also leaves a child link's external referent untouched.
#[test]
fn symlink_entries() {
    let fixture = Fixture::new();
    fixture.file("real");
    fixture.directory("directory");
    fixture.file("directory/child");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("real")).unwrap();
    let child = buffers.visit(&fixture.path("directory/child")).unwrap();
    insert(buffers.get_mut(id).unwrap(), "!");
    let prior = buffers.get(id).unwrap().file().unwrap().clone();
    for (name, target) in [
        ("file-link", "real"),
        ("dir-link", "directory"),
        ("dangling", "absent"),
    ] {
        symlink(target, fixture.path(name)).unwrap();
        assert!(
            buffers
                .delete(confirmed(Deletion::capture(&fixture.path(name)).unwrap()))
                .is_complete()
        );
        assert!(fs::symlink_metadata(fixture.path(name)).is_err());
    }
    fixture.directory("remove");
    symlink("../directory", fixture.path("remove/external")).unwrap();
    assert!(
        buffers
            .delete(recursive(&fixture.path("remove")))
            .is_complete()
    );
    assert_eq!(buffers.get(id).unwrap().file(), Some(&prior));
    assert!(buffers.get(child).unwrap().file().unwrap().is_present());
    assert_eq!(text(buffers.get(id).unwrap()), "!retained");
    assert_eq!(
        fs::read(fixture.path("directory/child")).unwrap(),
        b"retained"
    );
    assert_eq!(fs::read(fixture.path("real")).unwrap(), b"retained");
}

// Real parent write denial preserves the selected entry and its association.
// A separate inaccessible-parent capture reports the path without mutation.
#[test]
fn denied_delete() {
    for directory in [false, true] {
        let fixture = Fixture::new();
        let path = if directory {
            fixture.directory("entry");
            fixture.file("entry/child");
            fixture.path("entry/child")
        } else {
            fixture.file("entry");
            fixture.path("entry")
        };
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&path).unwrap();
        insert(buffers.get_mut(id).unwrap(), "!");
        let prior = buffers.get(id).unwrap().file().unwrap().clone();
        let deletion = if directory {
            recursive(&fixture.path("entry"))
        } else {
            confirmed(Deletion::capture(&fixture.path("entry")).unwrap())
        };
        // For recursion deny removing children as well as the parent entry.
        let blocked = if directory {
            fixture.path("entry")
        } else {
            fixture.0.clone()
        };
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o500)).unwrap();
        let result = buffers.delete(deletion);
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(result.error().unwrap().kind(), ErrorKind::PermissionDenied);
        assert_eq!(buffers.get(id).unwrap().file(), Some(&prior));
        assert_eq!(text(buffers.get(id).unwrap()), "!retained");
        assert_eq!(fs::read(&path).unwrap(), b"retained");
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o000)).unwrap();
        let result = Deletion::capture(&fixture.path("entry"));
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o700)).unwrap();
        let error = result.err().unwrap();
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("entry"));
    }
}

// Replacing a captured source invalidates both first and recursive decisions;
// publication rechecks the same identity even after an affirmative decision.
#[test]
fn stale_delete() {
    for stage in ["first", "recursive", "publish"] {
        let fixture = Fixture::new();
        let path = fixture.path("entry");
        let deletion = if stage == "recursive" {
            fixture.directory("entry");
            fixture.file("entry/child");
            let DeleteDecision::Recursive(deletion) =
                Deletion::capture(&path).unwrap().confirm().unwrap()
            else {
                panic!();
            };
            deletion
        } else {
            fixture.file("entry");
            let deletion = Deletion::capture(&path).unwrap();
            if stage == "publish" {
                confirmed(deletion)
            } else {
                deletion
            }
        };
        fs::rename(&path, fixture.path("held")).unwrap();
        fs::write(&path, b"replacement").unwrap();
        let error = if stage == "publish" {
            Buffers::new()
                .unwrap()
                .delete(deletion)
                .error()
                .unwrap()
                .to_string()
        } else {
            deletion.confirm().err().unwrap().to_string()
        };
        assert!(error.contains("selected entry changed"));
        assert_eq!(fs::read(path).unwrap(), b"replacement");
        assert!(fixture.path("held").exists());
    }
    assert!(Deletion::capture(std::path::Path::new("/")).is_err());
}

// Linux recursive removal can publish some child removals before permission
// failure. Use the observed directory order to block the last child, then check
// actual absences, retained text and refresh rather than claiming rollback.
#[test]
fn partial_delete() {
    let fixture = Fixture::new();
    fixture.directory("parent");
    for index in 0..8 {
        fixture.directory(&format!("parent/d{index}"));
        fixture.file(&format!("parent/d{index}/file"));
    }
    let children: Vec<_> = fs::read_dir(fixture.path("parent"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    let mut buffers = Buffers::new().unwrap();
    let ids: Vec<_> = children
        .iter()
        .map(|path| buffers.visit(&path.join("file")).unwrap())
        .collect();
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    choose(&mut tree, &fixture.path("parent"));
    tree.expand().unwrap();
    let deletion = recursive(&fixture.path("parent"));
    let blocked = children.last().unwrap();
    fs::set_permissions(blocked, fs::Permissions::from_mode(0o500)).unwrap();
    let result = buffers.delete(deletion);
    let refresh = tree.refresh();
    fs::set_permissions(blocked, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(result.error().unwrap().kind(), ErrorKind::PermissionDenied);
    refresh.unwrap();
    let removed = children.iter().filter(|path| !path.exists()).count();
    assert!(
        removed > 0 && removed < children.len(),
        "fixture must exercise actual partial deletion"
    );
    for (id, path) in ids.into_iter().zip(children) {
        let buffer = buffers.get(id).unwrap();
        assert_eq!(text(buffer), "retained");
        assert_eq!(
            buffer.file().unwrap().is_present(),
            path.join("file").exists()
        );
        assert_eq!(buffer.is_modified(), !path.exists());
        assert_eq!(
            tree.entries().iter().any(|entry| entry.path() == path),
            path.exists()
        );
    }
    assert_eq!(tree.selected().unwrap().path(), fixture.path("parent"));
}

// A new external occupant after deletion receives fresh overwrite protection.
// Refusing that save cannot destroy either the retained rope or external bytes.
#[test]
fn external_recreation() {
    let fixture = Fixture::new();
    fixture.file("file");
    let path = fixture.path("file");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    assert!(
        buffers
            .delete(confirmed(Deletion::capture(&path).unwrap()))
            .is_complete()
    );
    fs::write(&path, b"external").unwrap();
    let SaveRequest::Confirm(confirmation) = buffers.request_save(id, None).unwrap() else {
        panic!("missing external-change decision");
    };
    assert!(
        confirmation
            .message()
            .contains("Destination changed on disk")
    );
    drop(confirmation);
    assert_eq!(fs::read(path).unwrap(), b"external");
    assert_eq!(text(buffers.get(id).unwrap()), "retained");
    assert!(!buffers.get(id).unwrap().file().unwrap().is_present());
}

// Removing the displayed root leaves a recoverable placeholder, not an empty
// invalid tree. Its visiting descendants remain live without recreated parents.
#[test]
fn deleted_root() {
    let fixture = Fixture::new();
    fixture.directory("root");
    fixture.file("root/file");
    let root = fixture.path("root");
    let mut tree = Tree::new(root.clone());
    tree.expand().unwrap();
    tree.move_selection(true);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&root.join("file")).unwrap();
    assert!(buffers.delete(recursive(&root)).is_complete());
    assert_eq!(tree.refresh().unwrap_err().kind(), ErrorKind::NotFound);
    assert_eq!(tree.selected().unwrap().path(), root);
    assert_eq!(tree.entries().len(), 1);
    assert!(!buffers.save(id).unwrap().is_complete());
    assert!(!root.exists());
    assert_eq!(text(buffers.get(id).unwrap()), "retained");
    fs::create_dir(&root).unwrap();
    tree.refresh().unwrap();
    tree.expand().unwrap();
    assert_eq!(tree.entries().len(), 1);
}
