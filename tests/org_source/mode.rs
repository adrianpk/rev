#[path = "../configuration/fixture.rs"]
mod fixture;

use rev::buffer::{Buffers, Mode};
use rev::config::{Snapshot, prepare};
use rev::org::{Limits, Scan};

use fixture::Fixture;

// Visiting uses the existing literal association resolver, including user rules
// and case sensitivity. Missing paths do not become files merely to select Org.
#[test]
fn visiting_mode() {
    let cases = [
        ("factory", "", "notes.org", "Org"),
        ("case", "", "notes.ORG", "Text"),
        ("disabled", "org.enabled = false", "notes.org", "Text"),
        (
            "override",
            "[files.associations]\n'*.org' = 'plain'",
            "notes.org",
            "Text",
        ),
        (
            "custom",
            "[files.associations]\n'*.tasks' = 'org'",
            "notes.tasks",
            "Org",
        ),
    ];
    for (name, config, filename, expected) in cases {
        let fixture = Fixture::new();
        let settings = prepare(&fixture.write("config.toml", config)).snapshot;
        let mut buffers = Buffers::with_settings(settings).unwrap();
        let path = fixture.0.join(filename);
        let id = buffers.visit(&path).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert_eq!(buffer.mode(), expected, "{name}");
        assert!(buffer.history().is_empty(), "{name}");
        assert!(!buffer.is_modified(), "{name}");
        assert!(buffer.org_index().is_none(), "{name}");
        assert!(!path.exists(), "{name}");
    }
}

// Explicit major-mode choices are idempotent and buffer-local. They survive a
// settings reapplication without rewriting source or creating undo history.
#[test]
fn explicit_modes() {
    let fixture = Fixture::new();
    let path = fixture.write("source.rs", "\u{feff}* 界\r\n#+FUTURE: keep");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(true);
    let before = (buffer.stamp(), buffer.state(), super::bytes(buffer));
    for _ in 0..2 {
        buffer.select_mode(Mode::Org).unwrap();
        assert_eq!(buffer.mode(), "Org");
        assert_eq!(buffer.language(&Snapshot::default()), "rust");
    }
    let index = super::index::complete(Scan::new(buffer, Limits::default()).unwrap());
    buffer.install_org_index(index).unwrap();
    buffer.select_mode(Mode::Text).unwrap();
    buffer.apply_settings(&Snapshot::default());
    assert_eq!(buffer.mode(), "Text");
    assert!(!buffer.is_org());
    assert!(buffer.org_index().is_none());
    assert!(buffer.history().is_empty());
    assert!(buffer.is_readonly());
    assert_eq!(
        (buffer.stamp(), buffer.state(), super::bytes(buffer)),
        before
    );
    assert_eq!(std::fs::read(path).unwrap(), before.2);
}

// Configuration disabling retires Org observations and rejects explicit entry.
// Re-enabling restores the retained explicit choice, not a guessed file policy.
#[test]
fn mode_availability() {
    let fixture = Fixture::new();
    let disabled = prepare(&fixture.write("off.toml", "org.enabled = false")).snapshot;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("* H\n").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(Mode::Org).unwrap();
    let before = buffer.stamp();
    buffer.apply_settings(&disabled);
    assert!(!buffer.is_org());
    assert!(buffer.select_mode(Mode::Org).is_err());
    assert_eq!(buffer.mode(), "Text");
    buffer.apply_settings(&Snapshot::default());
    assert!(buffer.is_org());
    assert_eq!(buffer.stamp(), before);
    assert!(buffer.history().is_empty());
}
