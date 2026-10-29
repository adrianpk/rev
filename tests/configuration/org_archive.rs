use super::fixture::Fixture;
use rev::config::prepare;
// Archive policy remains literal data at startup, with finite path/heading
// bounds, per-leaf include resets and automatic standard archive recognition.
#[test]
fn archive_policy() {
    let f = Fixture::new();
    for (name, value, accepted) in [
        ("default", "%s_archive::".into(), true),
        ("heading", "missing/archive.org::* Done".into(), true),
        ("url", "https://example.test/archive::".into(), false),
        ("missing separator", "archive.org".into(), false),
        ("unsupported escape", "%x_archive::".into(), false),
        (
            "unsupported datetree",
            "archive.org::datetree/".into(),
            false,
        ),
        ("over limit", format!("{}::", "a".repeat(4095)), false),
    ] {
        let startup = prepare(&f.write("config.toml", &format!("org.archive.location='{value}'")));
        assert_eq!(
            startup.diagnostic.is_none(),
            accepted,
            "{name}: {:?}",
            startup.diagnostic
        );
        assert!(!f.0.join("missing").exists());
    }
    f.write(
        "base.toml",
        "org.archive.location='custom::'\norg.agenda.include_archived=true",
    );
    let startup = prepare(&f.write(
        "config.toml",
        "include=['base.toml']\norg.agenda.include_archived=false",
    ));
    assert!(startup.diagnostic.is_none());
    assert_eq!(startup.snapshot.settings().org.archive.location, "custom::");
    assert!(!startup.snapshot.settings().org.agenda.include_archived);
    assert_eq!(
        startup
            .snapshot
            .language(std::path::Path::new("notes.org_archive")),
        "org"
    );
}
