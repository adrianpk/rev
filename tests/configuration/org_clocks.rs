use super::fixture::Fixture;
use rev::config::{Snapshot, prepare};

// Clock policy keeps exact include provenance, including false/empty overrides;
// source placement and retention consume the effective values in native tests.
#[test]
fn clock_layers() {
    let fixture = Fixture::new();
    let included = fixture.write(
        "base.toml",
        "org.clock.drawer='WORK'\norg.clock.remove_zero=true\norg.clock.persist=true\norg.clock.history_limit=256",
    );
    let root = fixture.write(
        "config.toml",
        "include=['base.toml']\norg.clock.remove_zero=false\norg.clock.out_when_done=false",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let clock = &startup.snapshot.settings().org.clock;
    assert_eq!(clock.drawer, "WORK");
    assert!(!clock.remove_zero);
    assert!(!clock.out_when_done);
    assert!(clock.persist);
    assert_eq!(clock.history_limit, 256);
    assert_eq!(
        startup.snapshot.origin(&["org", "clock", "persist"]),
        Some(included.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["org", "clock", "drawer"]),
        Some(included.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["org", "clock", "remove_zero"]),
        Some(root.as_path())
    );
    let clock = Snapshot::default().settings().org.clock.clone();
    assert_eq!(clock.drawer, "LOGBOOK");
    assert!(clock.out_when_done);
    assert!(!clock.persist);
    assert_eq!(clock.history_limit, 32);
    assert!(!clock.remove_zero);
}

// Names cannot inject source delimiters. Unknown and mistyped fields reject
// the complete candidate while empty placement and the 64-byte limit admit.
#[test]
fn clock_bounds() {
    let fixture = Fixture::new();
    for (body, valid) in [
        ("drawer=''".into(), true),
        (format!("drawer='{}'", "A".repeat(64)), true),
        (format!("drawer='{}'", "A".repeat(65)), false),
        ("drawer='END'".into(), false),
        ("drawer='PROPERTIES'".into(), false),
        ("drawer='a:b'".into(), false),
        ("drawer=true".into(), false),
        ("remove_zero='false'".into(), false),
        ("history_limit=0".into(), true),
        ("history_limit=256".into(), true),
        ("history_limit=257".into(), false),
        ("history_limit=-1".into(), false),
        ("persist='true'".into(), false),
        ("unknown=false".into(), false),
    ] {
        let path = fixture.write("config.toml", &format!("[org.clock]\n{body}"));
        let startup = prepare(&path);
        assert_eq!(
            startup.diagnostic.is_none(),
            valid,
            "{body}: {:?}",
            startup.diagnostic
        );
        if !valid {
            assert_eq!(startup.snapshot, Snapshot::default());
        }
    }
}
