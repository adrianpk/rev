use super::fixture::Fixture;
use rev::config::prepare;
// Target policy is one typed leaf: invalid or over-limit paths reject at
// configuration admission, and an empty later array resets explicit roots.
#[test]
fn refile_policy() {
    let f = Fixture::new();
    for (name, value, accepted) in [
        ("empty", "[]".into(), true),
        ("local", "['missing/notes.org']".into(), true),
        ("wrong type", "true".into(), false),
        ("url", "['https://example.test/notes.org']".into(), false),
        ("empty path", "['']".into(), false),
        ("path over", format!("['{}']", "x".repeat(4097)), false),
        (
            "roots over",
            format!("[{}]", vec!["'x'"; 4097].join(",")),
            false,
        ),
    ] {
        let startup = prepare(&f.write("config.toml", &format!("org.refile.targets={value}")));
        assert_eq!(
            startup.diagnostic.is_none(),
            accepted,
            "{name}: {:?}",
            startup.diagnostic
        );
        assert!(!f.0.join("missing").exists());
    }
    f.write("base.toml", "org.refile.targets=['notes.org']");
    let startup = prepare(&f.write(
        "config.toml",
        "include=['base.toml']\norg.refile.targets=[]",
    ));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert!(startup.snapshot.settings().org.refile.targets.is_empty());
}
