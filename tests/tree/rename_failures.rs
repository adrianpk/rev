use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::{PermissionsExt, symlink};

use rev::buffer::{Buffers, SaveRequest};
use rev::file::{Publication, Rename};
use rev::tree::Tree;

use super::super::{Fixture, choose};
use super::{insert, text};

// Equal basenames keep distinct live labels and canonical owners. A rename
// into another buffer's display basename allocates a unique label, not its path.
#[test]
fn equal_basenames() {
    let fixture = Fixture::new();
    fixture.directory("a");
    fixture.directory("b");
    fixture.directory("c");
    fixture.file("a/same.txt");
    fixture.file("b/same.txt");
    fixture.file("c/new.txt");
    let mut buffers = Buffers::new().unwrap();
    let a = buffers.visit(&fixture.path("a/same.txt")).unwrap();
    let b = buffers.visit(&fixture.path("b/same.txt")).unwrap();
    let c = buffers.visit(&fixture.path("c/new.txt")).unwrap();
    let b_label = buffers.get(b).unwrap().name().to_owned();
    insert(buffers.get_mut(a).unwrap(), "!");
    buffers
        .rename(
            Rename::capture(&fixture.path("a/same.txt")).unwrap(),
            "new.txt",
        )
        .unwrap();
    assert_eq!(buffers.get(a).unwrap().name(), "new.txt<2>");
    assert_eq!(buffers.get(b).unwrap().name(), b_label);
    assert_eq!(buffers.get(c).unwrap().name(), "new.txt");
    assert_eq!(buffers.visit(&fixture.path("a/new.txt")).unwrap(), a);
    assert!(buffers.save(a).unwrap().is_complete());
    assert_eq!(fs::read(fixture.path("a/new.txt")).unwrap(), b"!retained");
    assert_eq!(fs::read(fixture.path("b/same.txt")).unwrap(), b"retained");
    assert_eq!(fs::read(fixture.path("c/new.txt")).unwrap(), b"retained");
}

// File/directory/dangling link entries move without following their targets.
// Canonical referent buffers keep their destinations, text and dirty histories.
#[test]
fn symlink_entries() {
    let fixture = Fixture::new();
    fixture.file("real");
    fixture.directory("directory");
    fixture.file("directory/child");
    symlink("real", fixture.path("file-link")).unwrap();
    symlink("directory", fixture.path("dir-link")).unwrap();
    symlink("absent", fixture.path("dangling")).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("file-link")).unwrap();
    let child = buffers.visit(&fixture.path("directory/child")).unwrap();
    insert(buffers.get_mut(id).unwrap(), "!");
    let stamp = buffers.get(id).unwrap().stamp();
    for (source, target, referent) in [
        ("file-link", "file-new", "real"),
        ("dir-link", "dir-new", "directory"),
        ("dangling", "dangling-new", "absent"),
    ] {
        buffers
            .rename(Rename::capture(&fixture.path(source)).unwrap(), target)
            .unwrap();
        assert_eq!(
            fs::read_link(fixture.path(target)).unwrap(),
            std::path::PathBuf::from(referent)
        );
        assert!(!fixture.path(source).exists());
    }
    assert_eq!(
        buffers.get(id).unwrap().file().unwrap().path(),
        fixture.path("real")
    );
    assert_eq!(
        buffers.get(child).unwrap().file().unwrap().path(),
        fixture.path("directory/child")
    );
    assert_eq!(buffers.get(id).unwrap().stamp(), stamp);
    assert_eq!(text(buffers.get(id).unwrap()), "!retained");
    assert_eq!(buffers.visit(&fixture.path("file-new")).unwrap(), id);
    assert_eq!(
        buffers.visit(&fixture.path("dir-new/child")).unwrap(),
        child
    );
    assert!(!fixture.path("absent").exists());
}

// Real parent write denial prevents both file and directory rename. Metadata
// and dirty text remain at the old path; restore permissions before assertions.
#[test]
fn denied_rename() {
    for directory in [false, true] {
        let fixture = Fixture::new();
        let file = if directory {
            fixture.directory("old");
            fixture.file("old/child");
            "old/child"
        } else {
            fixture.file("old");
            "old"
        };
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&fixture.path(file)).unwrap();
        insert(buffers.get_mut(id).unwrap(), "!");
        let stamp = buffers.get(id).unwrap().stamp();
        let rename = Rename::capture(&fixture.path("old")).unwrap();
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o500)).unwrap();
        let result = buffers.rename(rename, "new");
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(result.err().unwrap().kind(), ErrorKind::PermissionDenied);
        assert_eq!(
            buffers.get(id).unwrap().file().unwrap().path(),
            fixture.path(file)
        );
        assert_eq!(buffers.get(id).unwrap().stamp(), stamp);
        assert_eq!(text(buffers.get(id).unwrap()), "!retained");
        assert!(fixture.path("old").exists());
        assert!(!fixture.path("new").exists());
    }
}

// Captured source identity and late target conflicts are rechecked. Replacing
// the source cannot rename a different entry under an old prompt's authority.
#[test]
fn stale_entries() {
    let fixture = Fixture::new();
    fixture.file("old");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("old")).unwrap();
    let rename = Rename::capture(&fixture.path("old")).unwrap();
    fs::rename(fixture.path("old"), fixture.path("held")).unwrap();
    fs::write(fixture.path("old"), b"replacement").unwrap();
    assert!(
        buffers
            .rename(rename, "new")
            .err()
            .unwrap()
            .to_string()
            .contains("selected entry changed")
    );
    let rename = Rename::capture(&fixture.path("old")).unwrap();
    fixture.file("new");
    assert_eq!(
        buffers.rename(rename, "new").err().unwrap().kind(),
        ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read(fixture.path("old")).unwrap(), b"replacement");
    assert_eq!(fs::read(fixture.path("held")).unwrap(), b"retained");
    assert_eq!(fs::read(fixture.path("new")).unwrap(), b"retained");
    assert_eq!(
        buffers.get(id).unwrap().file().unwrap().path(),
        fixture.path("old")
    );
    assert_eq!(text(buffers.get(id).unwrap()), "retained");
}

// Renaming a modified-on-disk file moves its name but grants no overwrite
// consent. Later save still requires the normal external-change confirmation.
#[test]
fn external_observation() {
    let fixture = Fixture::new();
    fixture.file("old");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("old")).unwrap();
    insert(buffers.get_mut(id).unwrap(), "!");
    fs::write(fixture.path("old"), b"external changed bytes").unwrap();
    buffers
        .rename(Rename::capture(&fixture.path("old")).unwrap(), "new")
        .unwrap();
    assert_eq!(
        buffers.get(id).unwrap().file().unwrap().path(),
        fixture.path("new")
    );
    let SaveRequest::Confirm(confirmation) = buffers.request_save(id, None).unwrap() else {
        panic!("external change must remain guarded");
    };
    assert!(
        confirmation
            .message()
            .contains("Destination changed on disk")
    );
    assert_eq!(confirmation.target(), fixture.path("new"));
    assert_eq!(
        fs::read(fixture.path("new")).unwrap(),
        b"external changed bytes"
    );
    assert_eq!(text(buffers.get(id).unwrap()), "!retained");
}

// A save captured before rename cannot recreate its obsolete destination.
// A fresh capture writes the current path and acknowledges the same text state.
#[test]
fn stale_save() {
    let fixture = Fixture::new();
    fixture.file("old");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("old")).unwrap();
    insert(buffers.get_mut(id).unwrap(), "!");
    let save = buffers.prepare_save(id, None).unwrap();
    buffers
        .rename(Rename::capture(&fixture.path("old")).unwrap(), "new")
        .unwrap();
    let outcome = buffers.publish_save(save);
    assert_eq!(outcome.publication(), Publication::NotPublished);
    assert!(!fixture.path("old").exists());
    assert_eq!(fs::read(fixture.path("new")).unwrap(), b"retained");
    assert!(buffers.save(id).unwrap().is_complete());
    assert_eq!(fs::read(fixture.path("new")).unwrap(), b"!retained");
}

// Directory destinations reject all unrelated live reservations underneath,
// without recursively scanning unvisited source files to discover collisions.
#[test]
fn reserved_directory() {
    let fixture = Fixture::new();
    fixture.directory("old");
    fixture.file("old/child");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("old/child")).unwrap();
    for (target, reserved) in [("new", "new"), ("new-dir", "new-dir/unvisited")] {
        let owner = buffers.visit(&fixture.path(reserved)).unwrap();
        assert_eq!(
            buffers
                .rename(Rename::capture(&fixture.path("old")).unwrap(), target)
                .err()
                .unwrap()
                .kind(),
            ErrorKind::AlreadyExists
        );
        assert_eq!(buffers.visit(&fixture.path(reserved)).unwrap(), owner);
        assert!(!fixture.path(target).exists());
    }
    assert!(fixture.path("old/child").exists());
    assert_eq!(
        buffers.get(id).unwrap().file().unwrap().path(),
        fixture.path("old/child")
    );
}

// Renaming the root rebases loaded expanded branches and their selected child;
// all visiting descendants follow the physical destination without new owners.
#[test]
fn renamed_root() {
    let fixture = Fixture::new();
    fixture.directory("root/sub");
    fixture.file("root/sub/item");
    let mut tree = Tree::new(fixture.path("root"));
    let rename = tree.prepare_rename().unwrap();
    tree.expand().unwrap();
    tree.move_selection(true);
    tree.expand().unwrap();
    tree.move_selection(true);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("root/sub/item")).unwrap();
    let renamed = buffers.rename(rename, "renamed-root").unwrap();
    tree.renamed(&renamed).unwrap();
    assert_eq!(tree.root(), Some(fixture.path("renamed-root").as_path()));
    assert_eq!(
        tree.selected().unwrap().path(),
        fixture.path("renamed-root/sub/item")
    );
    assert!(tree.entries()[0].expanded());
    assert!(tree.entries()[1].expanded());
    assert_eq!(
        buffers
            .visit(&fixture.path("renamed-root/sub/item"))
            .unwrap(),
        id
    );
    assert!(!fixture.path("root").exists());
}

// A tree read failure after disk success must not reverse the new association
// or disguise publication. The renamed selection remains in the usable branch.
#[test]
fn renamed_refresh() {
    let fixture = Fixture::new();
    fixture.file("old");
    fixture.directory("blocked");
    fixture.file("blocked/child");
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    choose(&mut tree, &fixture.path("blocked"));
    tree.expand().unwrap();
    choose(&mut tree, &fixture.path("old"));
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("old")).unwrap();
    let renamed = buffers
        .rename(tree.prepare_rename().unwrap(), "new")
        .unwrap();
    fs::set_permissions(fixture.path("blocked"), fs::Permissions::from_mode(0o000)).unwrap();
    let result = tree.renamed(&renamed);
    fs::set_permissions(fixture.path("blocked"), fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
    assert!(fixture.path("new").exists());
    assert!(!fixture.path("old").exists());
    assert_eq!(tree.selected().unwrap().path(), fixture.path("new"));
    assert_eq!(
        buffers.get(id).unwrap().file().unwrap().path(),
        fixture.path("new")
    );
}
