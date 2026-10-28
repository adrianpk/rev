use super::fixture::Fixture;
use rev::config::{Snapshot, prepare};

// The existing loader retains Boolean presence and exact include provenance.
// Unknown/wrongly typed leaves reject the whole candidate without partial setup.
#[test]
fn tag_policy_layers() {
    let fixture = Fixture::new();
    let included = fixture.write("base.toml", "org.tags.inherit = false");
    for (name, body, expected) in [
        ("inherit", "", false),
        ("override", "org.tags.inherit = true", true),
    ] {
        let root = fixture.write("config.toml", &format!("include = ['base.toml']\n{body}"));
        let startup = prepare(&root);
        assert!(
            startup.diagnostic.is_none(),
            "{name}: {:?}",
            startup.diagnostic
        );
        assert_eq!(startup.snapshot.settings().org.tags.inherit, expected);
        assert_eq!(
            startup.snapshot.origin(&["org", "tags", "inherit"]),
            Some(if body.is_empty() {
                included.as_path()
            } else {
                root.as_path()
            })
        );
    }
    for body in [
        "org.tags.inherit = ['Work']",
        "org.tags.inherit = 'true'",
        "org.tags.unknown = true",
    ] {
        let root = fixture.write("config.toml", body);
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_some());
        assert_eq!(startup.snapshot, Snapshot::default());
    }
}
