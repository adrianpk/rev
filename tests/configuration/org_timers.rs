use super::fixture::Fixture;
use rev::config::{Snapshot, prepare};
// Countdown defaults are typed data with exact layer provenance; parsing them
// never arms or restores session state and invalid candidates reject completely.
#[test]
fn timer_defaults() {
    let fixture = Fixture::new();
    fixture.write("base.toml", "org.timer.default='2'");
    let root = fixture.write(
        "config.toml",
        "include=['base.toml']\norg.timer.default='0:00:03'",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    assert_eq!(startup.snapshot.settings().org.timer.default, "0:00:03");
    assert_eq!(
        startup.snapshot.origin(&["org", "timer", "default"]),
        Some(root.as_path())
    );
    assert_eq!(Snapshot::default().settings().org.timer.default, "0");
    for (name, value, admit) in [
        ("zero", "'0'", true),
        ("limit", "'8760:00:00'", true),
        ("over", "'8760:00:01'", false),
        ("type", "3", false),
        ("negative", "'-1'", false),
        ("bad", "'1:99:00'", false),
    ] {
        let path = fixture.write("candidate.toml", &format!("org.timer.default={value}"));
        assert_eq!(prepare(&path).diagnostic.is_none(), admit, "{name}");
    }
}
