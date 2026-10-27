use rev::config::prepare;

use super::*;

fn configured(fixture: &Fixture) -> Buffers {
    let path = fixture.path("config.toml");
    fs::write(
        &path,
        b"language.go.tab_width = 8\nlanguage.rust.tab_width = 2",
    )
    .unwrap();
    let startup = prepare(&path);
    assert!(startup.diagnostic.is_none());
    fs::write(path, b"language.go.tab_width = 3").unwrap();
    Buffers::with_settings(startup.snapshot).unwrap()
}

// Successful file renames resolve from the frozen startup snapshot, while
// failed renames leave policy, file identity and dirty saved-state truth intact.
#[test]
fn file_policy() {
    let fixture = Fixture::new();
    fixture.file("old.txt");
    fixture.file("occupied.go");
    let mut buffers = configured(&fixture);
    let id = buffers.visit(&fixture.path("old.txt")).unwrap();
    insert(buffers.get_mut(id).unwrap(), "!");
    let buffer = buffers.get(id).unwrap();
    let before = (buffer.stamp(), buffer.state());
    assert!(
        buffers
            .rename(
                Rename::capture(&fixture.path("old.txt")).unwrap(),
                "occupied.go"
            )
            .is_err()
    );
    assert_eq!(buffers.get(id).unwrap().policy().tab_width, 4);
    let mut prior = "old.txt";
    for (name, width) in [("new.go", 8), ("next.rs", 2)] {
        let renamed = buffers
            .rename(Rename::capture(&fixture.path(prior)).unwrap(), name)
            .unwrap();
        assert!(renamed.warning().is_none());
        let buffer = buffers.get(id).unwrap();
        assert_eq!(buffer.policy().tab_width, width);
        assert_eq!((buffer.stamp(), buffer.state()), before);
        assert_eq!(buffer.history().len(), 1);
        assert!(buffer.is_modified());
        assert_eq!(text(buffer), "!retained");
        assert_eq!(buffer.file().unwrap().path(), fixture.path(name));
        assert_eq!(buffer.file().unwrap().basename(), name);
        assert_eq!(fs::read(fixture.path(name)).unwrap(), b"retained");
        assert_eq!(buffers.visit(&fixture.path(name)).unwrap(), id);
        prior = name;
    }
}

// A directory move updates every visited descendant's path while retaining
// its basename policy, alias identity, format and clean/dirty history. Similar
// sibling prefixes and pending new files keep their own ownership boundaries.
#[test]
fn directory_policy() {
    use rev::buffer::edit::History;
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    fixture.directory("old/sub");
    fixture.directory("old-other");
    for name in [
        "old/a.go",
        "old/sub/b.rs",
        "old/sub/body.txt",
        "old-other/a.go",
    ] {
        fs::write(fixture.path(name), b"\xef\xbb\xbf\tA\r\nlast").unwrap();
    }
    symlink("sub/body.txt", fixture.path("old/alias.go")).unwrap();
    let mut buffers = configured(&fixture);
    let mut cases = Vec::new();
    for (prior, target, width) in [
        ("old/a.go", "moved/a.go", 8),
        ("old/sub/b.rs", "moved/sub/b.rs", 2),
        ("old/alias.go", "moved/sub/body.txt", 8),
        ("old/sub/new.go", "moved/sub/new.go", 8),
        ("old-other/a.go", "old-other/a.go", 8),
    ] {
        let id = buffers.visit(&fixture.path(prior)).unwrap();
        let buffer = buffers.get(id).unwrap();
        cases.push((id, target, width, buffer.stamp(), buffer.state()));
    }
    let dirty = cases[0].0;
    insert(buffers.get_mut(dirty).unwrap(), "!");
    cases[0].3 = buffers.get(dirty).unwrap().stamp();
    cases[0].4 = buffers.get(dirty).unwrap().state();
    let renamed = buffers
        .rename(Rename::capture(&fixture.path("old")).unwrap(), "moved")
        .unwrap();
    assert!(renamed.warning().is_none());
    for (id, target, width, stamp, state) in cases {
        let buffer = buffers.get(id).unwrap();
        assert_eq!(buffer.policy().tab_width, width);
        assert_eq!((buffer.stamp(), buffer.state()), (stamp, state));
        assert_eq!(buffer.is_modified(), id == dirty);
        assert_eq!(buffer.history().len(), usize::from(id == dirty));
        assert_eq!(buffer.file().unwrap().path(), fixture.path(target));
        assert_eq!(buffers.visit(&fixture.path(target)).unwrap(), id);
    }
    let alias = buffers.visit(&fixture.path("moved/alias.go")).unwrap();
    assert_eq!(
        buffers.get(alias).unwrap().file().unwrap().basename(),
        "alias.go"
    );
    assert_eq!(buffers.get(alias).unwrap().policy().tab_width, 8);
    let buffer = buffers.get_mut(dirty).unwrap();
    assert!(buffer.has_bom());
    assert_eq!(buffer.newline(), "\r\n");
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert!(!buffer.is_modified());
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert!(buffer.is_modified());
    assert_eq!(text(buffer), "!\tA\r\nlast");
    assert!(matches!(
        buffers.request_save(dirty, None).unwrap(),
        SaveRequest::Ready(_)
    ));
    assert!(!fixture.path("moved/sub/new.go").exists());
    assert_eq!(
        fs::read(fixture.path("moved/a.go")).unwrap(),
        b"\xef\xbb\xbf\tA\r\nlast"
    );
}
