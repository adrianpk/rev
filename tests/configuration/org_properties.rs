use super::fixture::Fixture;
use rev::config::{Snapshot, parse, prepare};

// Leaf presence and array replacement survive include layering with source
// provenance. Explicit empty disables inheritance without resetting Org mode.
#[test]
fn property_policy_layers() {
    let fixture = Fixture::new();
    let included = fixture.write("base.toml", "org.properties.inherit = ['X', 'UNKNOWN']");
    for (name, body, expected) in [
        ("inherit", "", vec!["X", "UNKNOWN"]),
        ("replace", "org.properties.inherit = ['Y']", vec!["Y"]),
        ("disable", "org.properties.inherit = []", vec![]),
    ] {
        let root = fixture.write("config.toml", &format!("include = ['base.toml']\n{body}"));
        let startup = prepare(&root);
        assert!(
            startup.diagnostic.is_none(),
            "{name}: {:?}",
            startup.diagnostic
        );
        assert_eq!(
            startup.snapshot.settings().org.properties.inherit,
            expected,
            "{name}"
        );
        assert!(startup.snapshot.settings().org.enabled);
        assert_eq!(
            startup.snapshot.origin(&["org", "properties", "inherit"]),
            Some(if body.is_empty() {
                included.as_path()
            } else {
                root.as_path()
            })
        );
    }
}

// Policy is typed and bounded, with case-insensitive identities. Invalid
// candidates fail atomically through the actual loader and preserve defaults.
#[test]
fn property_policy_admission() {
    let fixture = Fixture::new();
    let many = (0..129)
        .map(|i| format!("'X{i}'"))
        .collect::<Vec<_>>()
        .join(",");
    for value in [
        "true".into(),
        "['X', 'x']".into(),
        "['X+']".into(),
        "['ITEM']".into(),
        "['END']".into(),
        "['has space']".into(),
        format!("['{}']", "x".repeat(129)),
        format!("[{many}]"),
    ] {
        let body = format!("org.properties.inherit = {value}");
        let root = fixture.write("config.toml", &body);
        let error = parse(&root, body.as_bytes()).unwrap_err();
        assert_eq!(error.key, ["org", "properties", "inherit"], "{value}");
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_some());
        assert_eq!(startup.snapshot, Snapshot::default());
    }
}
