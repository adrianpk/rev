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
