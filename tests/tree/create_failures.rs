use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::{PermissionsExt, symlink};

use rev::tree::CreateKind;

use super::super::{Fixture, choose};

// A prepared request retains its chosen directory even if the visible
// selection changes before publication; reconciliation selects the real child.
#[test]
fn captured_directory() {
    let fixture = Fixture::new();
    fixture.directory("a");
    fixture.directory("b");
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    tree.move_selection(true);
    let creation = tree.prepare_create(CreateKind::File).unwrap();
    tree.move_selection(true);
    let path = creation.publish("new.txt").unwrap();
    assert_eq!(path, fixture.path("a/new.txt"));
    assert!(!fixture.path("b/new.txt").exists());
    tree.reveal_created(&path).unwrap();
    assert_eq!(tree.selected().unwrap().path(), path);
    assert_eq!(tree.entries().len(), 4);
}

// Publication revalidates the captured parent. Deletion or replacement with
// a file or symlink cannot redirect creation or write through an old selection.
#[test]
fn parent_changes() {
    for replacement in ["missing", "file", "symlink"] {
        let fixture = Fixture::new();
        fixture.directory("a");
        fixture.directory("outside");
        let mut tree = fixture.tree();
        tree.expand().unwrap();
        tree.move_selection(true);
        let creation = tree.prepare_create(CreateKind::File).unwrap();
        fs::remove_dir(fixture.path("a")).unwrap();
        match replacement {
            "file" => fixture.file("a"),
            "symlink" => symlink(fixture.path("outside"), fixture.path("a")).unwrap(),
            _ => {}
        }
        let error = creation.publish("new.txt").unwrap_err();
        let expected = if replacement == "missing" {
            ErrorKind::NotFound
        } else {
            ErrorKind::InvalidInput
        };
        assert_eq!(error.kind(), expected, "{replacement}");
        assert!(!fixture.path("outside/new.txt").exists());
    }
}

// Permission failure occurs at the real exclusive creation boundary. Restore
// fixture permissions before asserting so cleanup is safe on ordinary failure.
#[test]
fn denied_creation() {
    for kind in [CreateKind::File, CreateKind::Directory] {
        let fixture = Fixture::new();
        let tree = fixture.tree();
        let creation = tree.prepare_create(kind).unwrap();
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o500)).unwrap();
        let result = creation.publish("denied");
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o700)).unwrap();
        let error = result.unwrap_err();
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("create entry"));
        assert!(!fixture.path("denied").exists());
    }
}

// File and symlink selections refuse creation rather than substituting their
// parent or link target as a hidden location. Existing bytes stay unchanged.
#[test]
fn leaf_selection() {
    let fixture = Fixture::new();
    fixture.directory("directory");
    fixture.file("file");
    symlink(fixture.path("directory"), fixture.path("link")).unwrap();
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    for name in ["file", "link"] {
        choose(&mut tree, &fixture.path(name));
        for kind in [CreateKind::File, CreateKind::Directory] {
            let result = tree.prepare_create(kind);
            assert!(matches!(result, Err(error) if error.kind() == ErrorKind::InvalidInput));
        }
    }
    assert_eq!(fs::read(fixture.path("file")).unwrap(), b"retained");
    assert_eq!(fs::read_dir(fixture.path("directory")).unwrap().count(), 0);
}

// A refresh failure after publication does not undo or conceal the new file.
// Accessible parent observations and new selection still reconcile correctly.
#[test]
fn published_creation() {
    let fixture = Fixture::new();
    fixture.directory("blocked");
    fixture.file("blocked/old");
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    tree.move_selection(true);
    tree.expand().unwrap();
    tree.move_selection(false);
    let path = tree
        .prepare_create(CreateKind::File)
        .unwrap()
        .publish("published")
        .unwrap();
    fs::set_permissions(fixture.path("blocked"), fs::Permissions::from_mode(0o000)).unwrap();
    let result = tree.reveal_created(&path);
    fs::set_permissions(fixture.path("blocked"), fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
    assert!(path.is_file());
    assert_eq!(tree.selected().unwrap().path(), path);
    assert_eq!(fs::read(&path).unwrap(), b"");
}

// A newly created unsupported extension enters the real file pipeline. A
// later canonical/alias visit reuses the dirty owner and its undo history.
#[test]
fn created_visit() {
    use rev::command::{Command, History};
    let fixture = Fixture::new();
    let mut tree = fixture.tree();
    let path = tree
        .prepare_create(CreateKind::File)
        .unwrap()
        .publish("new.unknown")
        .unwrap();
    tree.reveal_created(&path).unwrap();
    let mut session = crate::editor();
    session.load(tree.selected().unwrap().path()).unwrap();
    let id = session.buffer().unwrap().id();
    session.insert('界');
    let stamp = session.buffer().unwrap().stamp();
    symlink(&path, fixture.path("alias")).unwrap();
    session.load(&fixture.path("alias")).unwrap();
    assert_eq!(session.buffer().unwrap().id(), id);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(crate::text(&session), "界");
    assert!(session.buffer().unwrap().is_modified());
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("Text"))
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(crate::text(&session), "");
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(fs::read(&path).unwrap(), b"");
}

// Names are not silently truncated to a kernel limit. A real filesystem
// rejection returns its path/operation and leaves no partial entry behind.
#[test]
fn long_name() {
    let fixture = Fixture::new();
    let tree = fixture.tree();
    let name = "x".repeat(256);
    for kind in [CreateKind::File, CreateKind::Directory] {
        let error = tree
            .prepare_create(kind)
            .unwrap()
            .publish(&name)
            .unwrap_err();
        assert!(error.to_string().contains("inspect destination"));
        assert!(error.to_string().contains(&name));
    }
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
}
