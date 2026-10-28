use super::fixture::Fixture;
use rev::config::{Snapshot, prepare};

// Only estimate leaves project during validation. Normal includes retain actual
// provenance while admitted unit overrides change the live duration consumer.
#[test]
fn estimate_layers() {
    let fixture = Fixture::new();
    let included = fixture.write(
        "base.toml",
        "[org.priority]\nhighest='D'\nlowest='F'\ndefault='E'\n[org.duration.units]\nd=480.0",
    );
    let root = fixture.write(
        "config.toml",
        "include=['base.toml']\norg.priority.default='F'\norg.duration.units.h=30.0",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let policy = &startup.snapshot.settings().org;
    assert_eq!(policy.priority.highest, "D");
    assert_eq!(policy.priority.default, "F");
    assert_eq!(
        rev::org::Duration::parse("1d2h", &policy.duration.units)
            .unwrap()
            .minutes(),
        540.0
    );
    assert_eq!(
        startup.snapshot.origin(&["org", "duration", "units", "d"]),
        Some(included.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["org", "priority", "default"]),
        Some(root.as_path())
    );
}

// Non-finite/nonpositive scales and ambiguous priority ranges reject the entire
// candidate, including malformed unrelated fields before their projection.
#[test]
fn estimate_rejection() {
    let fixture = Fixture::new();
    for source in [
        "org.priority.highest='Z'",
        "org.priority.lowest='A'",
        "org.priority.default='AA'",
        "org.priority.default='2'",
        "org.priority.default='a'",
        "org.duration.units.d=0.0",
        "org.duration.units.h=-1.0",
        "org.duration.units.min=inf",
        "org.duration.units.y=nan",
        "org.duration.units.unknown=1.0",
        "palette.foreground='bad'\norg.priority.default='C'",
    ] {
        let root = fixture.write("config.toml", source);
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_some(), "{source}");
        assert_eq!(startup.snapshot, Snapshot::default());
    }
}
