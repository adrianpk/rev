use std::io::ErrorKind;
use std::os::unix::fs::{PermissionsExt, symlink};

use rev::buffer::edit::History;
use rev::file::{DeleteDecision, Deletion, Rename};
use rev::tree::Tree;

use super::*;

fn retained(fixture: &Fixture) -> (Buffers, rev::buffer::BufferId) {
    let path = fixture.file(
        "source.rs",
        "\u{feff}// 界e\u{301}🙂\r\nfn base() {}".as_bytes(),
    );
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    edit(buffer, "// retained\r\n");
    edit(buffer, "// redo\r\n");
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    buffer.refresh_syntax();
    assert_eq!(buffer.mode(), "Rust");
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(buffer.redo_history().len(), 1);
    (buffers, id)
}

// Cyclic paths and genuine permission denial fail before publishing a visit.
// Repeated tree refresh/expansion and failed ingress retain both history arms,
// exact BOM/CRLF bytes and current syntax; later real save still succeeds.
#[test]
fn failed_paths() {
    let fixture = Fixture::new();
    let (mut buffers, id) = retained(&fixture);
    symlink("cycle-b", fixture.0.join("cycle-a")).unwrap();
    symlink("cycle-a", fixture.0.join("cycle-b")).unwrap();
    let denied = fixture.0.join("denied");
    fs::create_dir(&denied).unwrap();
    fs::write(denied.join("child.rs"), b"retained").unwrap();
    let mut tree = Tree::new(fixture.0.clone());
    tree.expand().unwrap();
    tree.move_selection(true);
    assert_eq!(tree.selected().unwrap().path(), denied);
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).unwrap();
    let result = tree.expand();
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o700)).unwrap();
    let failure = result.unwrap_err();
    assert_eq!(failure.kind(), ErrorKind::PermissionDenied);
    assert!(!tree.selected().unwrap().expanded());
    assert_eq!(tree.entries().len(), 5);
    let before = snapshot(buffers.get(id).unwrap());
    let captures = buffers.get(id).unwrap().syntax().captures(
        buffers.get(id).unwrap(),
        0..buffers.get(id).unwrap().len_bytes(),
    );
    for name in ["cycle-a", "cycle-b", "denied/child.rs"] {
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).unwrap();
        let result = buffers.visit(&fixture.0.join(name));
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o700)).unwrap();
        let failure = result.unwrap_err();
        if name.starts_with("denied") {
            assert_eq!(failure.kind(), ErrorKind::PermissionDenied);
        } else {
            assert_eq!(
                failure.kind(),
                fs::canonicalize(fixture.0.join(name)).unwrap_err().kind()
            );
        }
        assert!(failure.to_string().contains(name));
        assert_eq!(snapshot(buffers.get(id).unwrap()), before);
        assert_eq!(buffers.iter().count(), 2);
    }
    for _ in 0..4 {
        for name in ["cycle-a", "cycle-b"] {
            let path = fixture.0.join(name);
            select(&mut tree, &path);
            assert!(tree.selected().unwrap().is_symlink());
            tree.expand().unwrap();
            tree.refresh().unwrap();
            assert_eq!(tree.entries().len(), 5);
        }
        tree.collapse();
        tree.collapse();
        tree.expand().unwrap();
    }
    let buffer = buffers.get(id).unwrap();
    assert_eq!(snapshot(buffer), before);
    assert_eq!(
        buffer.syntax().captures(buffer, 0..buffer.len_bytes()),
        captures
    );
    assert!(buffers.save(id).unwrap().is_complete());
    assert_eq!(fs::read(fixture.0.join("source.rs")).unwrap(), before.bytes);
    let buffer = buffers.get_mut(id).unwrap();
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert!(buffer.is_modified());
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert!(!buffer.is_modified());
}

// Canceling either real deletion stage and a rename draft cannot consume a
// visiting buffer's redo, logical state, syntax or BOM/EOF reservation. A real
// occupied-target refusal likewise leaves both source and destination intact.
#[test]
fn canceled_tree() {
    let fixture = Fixture::new();
    let (mut buffers, id) = retained(&fixture);
    let source = fixture.0.join("source.rs");
    let disk = fs::read(&source).unwrap();
    let occupied = fixture.file("occupied.rs", b"unrelated");
    let before = snapshot(buffers.get(id).unwrap());
    for recursive in [false, true] {
        let deletion = Deletion::capture(&fixture.0).unwrap();
        if recursive {
            let DeleteDecision::Recursive(deletion) = deletion.confirm().unwrap() else {
                panic!("nonempty directory must require its second decision");
            };
            drop(deletion);
        } else {
            drop(deletion);
        }
        drop(Rename::capture(&source).unwrap());
        assert_eq!(snapshot(buffers.get(id).unwrap()), before);
        assert_eq!(fs::read(&source).unwrap(), disk);
        assert_eq!(fs::read(&occupied).unwrap(), b"unrelated");
    }
    let error = buffers
        .rename(Rename::capture(&source).unwrap(), "occupied.rs")
        .err()
        .unwrap();
    assert_eq!(error.kind(), ErrorKind::AlreadyExists);
    assert_eq!(snapshot(buffers.get(id).unwrap()), before);
    assert_eq!(fs::read(&source).unwrap(), disk);
    assert_eq!(fs::read(&occupied).unwrap(), b"unrelated");
    let buffer = buffers.get_mut(id).unwrap();
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    let mut expected = before;
    expected.stamp = buffer.stamp();
    assert_eq!(snapshot(buffer), expected);
}

fn select(tree: &mut Tree, path: &Path) {
    for _ in 0..tree.entries().len() {
        if tree.selected().unwrap().path() == path {
            break;
        }
        tree.move_selection(true);
    }
    assert_eq!(tree.selected().unwrap().path(), path);
}
