use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;

use super::{Fixture, choose, selected};

struct Locked(PathBuf);
impl Locked {
    fn new(path: PathBuf) -> Self {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        Self(path)
    }
}
impl Drop for Locked {
    fn drop(&mut self) {
        fs::set_permissions(&self.0, fs::Permissions::from_mode(0o700)).unwrap();
    }
}

// Each step exposes just one additional level. Refresh visits only those open
// levels, with no recursive tree algorithm or discovery beyond the user's work.
#[test]
fn deep_branches() {
    let fixture = Fixture::new();
    let mut path = fixture.0.clone();
    for _ in 0..200 {
        path.push("n");
        fs::create_dir(&path).unwrap();
    }
    fs::write(path.join("end"), b"retained").unwrap();
    let mut tree = fixture.tree();
    for depth in 0..200 {
        assert_eq!(tree.selected().unwrap().depth(), depth);
        tree.expand().unwrap();
        assert_eq!(tree.entries().len(), depth + 2);
        tree.move_selection(true);
    }
    assert_eq!(selected(&tree), path);
    tree.expand().unwrap();
    tree.move_selection(true);
    assert_eq!(tree.entries().len(), 202);
    tree.refresh().unwrap();
    assert_eq!(selected(&tree), path.join("end"));
    tree.collapse();
    tree.collapse();
    assert_eq!(selected(&tree), path);
    assert_eq!(tree.entries().len(), 201);
}

// Directory links, parent/self cycles, chains and dangling links are equally
// leaves. Neither expansion nor refresh follows their targets.
#[test]
fn symlink_leaves() {
    let fixture = Fixture::new();
    fixture.directory("real/deep");
    for (name, target) in [
        ("self", fixture.0.clone()),
        ("parent", fixture.0.parent().unwrap().to_path_buf()),
        ("chain", fixture.path("self")),
        ("missing", fixture.path("absent")),
    ] {
        symlink(target, fixture.path(name)).unwrap();
    }
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    for name in ["self", "parent", "chain", "missing"] {
        choose(&mut tree, &fixture.path(name));
        assert!(tree.selected().unwrap().is_symlink());
        tree.expand().unwrap();
        assert_eq!(tree.entries().len(), 6);
        assert!(
            tree.activate()
                .unwrap_err()
                .to_string()
                .contains("entry is a leaf")
        );
    }
    tree.refresh().unwrap();
    assert_eq!(tree.entries().len(), 6);
    assert!(tree.entries().iter().all(|entry| entry.depth() <= 1));
}

// A failed direct read publishes nothing. Refresh collapses only the failed
// open branch and still reloads accessible siblings; its error names the path.
#[test]
fn denied_branch() {
    let fixture = Fixture::new();
    fixture.directory("a");
    fixture.directory("b");
    fixture.file("a/old");
    fixture.file("b/old");
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    tree.move_selection(true);
    tree.expand().unwrap();
    choose(&mut tree, &fixture.path("b"));
    tree.expand().unwrap();
    choose(&mut tree, &fixture.path("a/old"));
    fixture.file("b/new");
    let locked = Locked::new(fixture.path("a"));
    let error = tree.refresh().unwrap_err();
    assert_eq!(error.kind(), ErrorKind::PermissionDenied);
    assert!(
        error
            .to_string()
            .contains(&format!("read directory {}", fixture.path("a").display()))
    );
    assert_eq!(selected(&tree), fixture.path("a"));
    assert!(!tree.selected().unwrap().expanded());
    assert!(
        tree.entries()
            .iter()
            .any(|entry| entry.path() == fixture.path("b/new"))
    );
    let count = tree.entries().len();
    assert_eq!(
        tree.expand().unwrap_err().kind(),
        ErrorKind::PermissionDenied
    );
    assert_eq!(tree.entries().len(), count);
    drop(locked);
    tree.expand().unwrap();
    assert!(tree.selected().unwrap().expanded());
}

// Deleted stale entries keep a usable selection until explicit reconciliation.
// Movement and parent transfer still work while the old observation is retained.
#[test]
fn deleted_directory() {
    let fixture = Fixture::new();
    fixture.directory("gone");
    fixture.file("keep");
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    tree.move_selection(true);
    fs::remove_dir(fixture.path("gone")).unwrap();
    let error = tree.expand().unwrap_err();
    assert_eq!(error.kind(), ErrorKind::NotFound);
    assert!(error.to_string().contains("inspect entry"));
    assert_eq!(selected(&tree), fixture.path("gone"));
    tree.move_selection(true);
    assert_eq!(selected(&tree), fixture.path("keep"));
    tree.collapse();
    tree.refresh().unwrap();
    assert_eq!(selected(&tree), fixture.0);
    assert_eq!(tree.entries().len(), 2);
}

// A missing root clears stale descendants, remains selected and collapsed,
// and can recover through refresh after the same root directory is recreated.
#[test]
fn missing_root() {
    let fixture = Fixture::new();
    fixture.file("old");
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    tree.move_selection(true);
    fs::remove_dir_all(&fixture.0).unwrap();
    assert_eq!(tree.refresh().unwrap_err().kind(), ErrorKind::NotFound);
    assert_eq!(tree.entries().len(), 1);
    assert_eq!(selected(&tree), fixture.0);
    assert!(!tree.selected().unwrap().expanded());
    fs::create_dir(&fixture.0).unwrap();
    fixture.file("new");
    tree.refresh().unwrap();
    tree.expand().unwrap();
    tree.move_selection(true);
    assert_eq!(selected(&tree), fixture.path("new"));
}

// Replacing a formerly expanded directory with a link must discard its stale
// descendants. A stale collapsed observation also revalidates before reading.
#[test]
fn replaced_directory() {
    let fixture = Fixture::new();
    fixture.directory("a");
    fixture.file("a/old");
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    tree.move_selection(true);
    fs::remove_dir_all(fixture.path("a")).unwrap();
    symlink(&fixture.0, fixture.path("a")).unwrap();
    assert!(
        tree.expand()
            .unwrap_err()
            .to_string()
            .contains("entry is not a directory")
    );
    assert_eq!(tree.entries().len(), 2);
    tree.refresh().unwrap();
    assert!(tree.selected().unwrap().is_symlink());
    assert!(!tree.selected().unwrap().is_directory());
    fs::remove_file(fixture.path("a")).unwrap();
    fixture.directory("a");
    fixture.file("a/old");
    tree.refresh().unwrap();
    tree.expand().unwrap();
    tree.move_selection(true);
    fs::remove_dir_all(fixture.path("a")).unwrap();
    symlink(&fixture.0, fixture.path("a")).unwrap();
    tree.refresh().unwrap();
    assert_eq!(selected(&tree), fixture.path("a"));
    assert!(tree.selected().unwrap().is_symlink());
    assert_eq!(tree.entries().len(), 2);
}

// External rename/add/remove changes become visible only on refresh. Renamed
// branches start collapsed, and old selection identity is never guessed by name.
#[test]
fn external_changes() {
    let fixture = Fixture::new();
    fixture.directory("a");
    fixture.file("a/old");
    fixture.file("gone");
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    tree.move_selection(true);
    tree.expand().unwrap();
    tree.move_selection(true);
    fs::rename(fixture.path("a"), fixture.path("z")).unwrap();
    fs::remove_file(fixture.path("gone")).unwrap();
    fixture.file("added");
    assert_eq!(selected(&tree), fixture.path("a/old"));
    tree.refresh().unwrap();
    assert_eq!(selected(&tree), fixture.0);
    assert_eq!(tree.entries().len(), 3);
    assert_eq!(tree.entries()[1].path(), fixture.path("z"));
    assert!(!tree.entries()[1].expanded());
    assert_eq!(tree.entries()[2].path(), fixture.path("added"));
}

// Native filename ordering and path identity retain distinct non-UTF8 names,
// even when their display replacement text would be identical.
#[test]
fn native_names() {
    let fixture = Fixture::new();
    let names = [
        OsString::from_vec(vec![b'x', 0xfe]),
        OsString::from_vec(vec![b'x', 0xff]),
    ];
    for name in names.iter().rev() {
        fs::write(fixture.0.join(name), b"exact").unwrap();
    }
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    let paths: Vec<_> = tree.entries()[1..]
        .iter()
        .map(|entry| entry.path().to_path_buf())
        .collect();
    assert_eq!(paths, names.map(|name| fixture.0.join(name)));
    tree.move_selection(true);
    tree.refresh().unwrap();
    assert_eq!(selected(&tree), paths[0]);
    for path in paths {
        assert_eq!(fs::read(path).unwrap(), b"exact");
    }
}
