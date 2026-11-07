use super::fixture::Fixture;
use rev::config::{Settings, StartupVisibility, prepare};

// Every policy is a real enum leaf, including an explicit compatibility default.
// Includes retain omitted siblings and the actual assigning source.
#[test]
fn policy_layers() {
    let f = Fixture::new();
    let base = f.write(
        "base.toml",
        "org.startup_folded='overview'\norg.indent=false",
    );
    for (name, later, expected) in [
        ("inherit", "", StartupVisibility::Overview),
        (
            "overview",
            "org.startup_folded='overview'",
            StartupVisibility::Overview,
        ),
        (
            "content",
            "org.startup_folded='content'",
            StartupVisibility::Content,
        ),
        (
            "showall",
            "org.startup_folded='showall'",
            StartupVisibility::Showall,
        ),
        (
            "showeverything",
            "org.startup_folded='showeverything'",
            StartupVisibility::Showeverything,
        ),
    ] {
        let root = f.write("config.toml", &format!("include=['base.toml']\n{later}"));
        let startup = prepare(&root);
        assert!(
            startup.diagnostic.is_none(),
            "{name}: {:?}",
            startup.diagnostic
        );
        assert_eq!(
            startup.snapshot.settings().org.startup_folded,
            expected,
            "{name}"
        );
        assert!(!startup.snapshot.settings().org.indent);
        assert_eq!(
            startup.snapshot.origin(&["org", "startup_folded"]),
            Some(if later.is_empty() {
                base.as_path()
            } else {
                root.as_path()
            })
        );
        assert_eq!(
            std::fs::read_to_string(&root).unwrap(),
            format!("include=['base.toml']\n{later}")
        );
    }
    let root = f.write("config.toml", "");
    let startup = prepare(&root);
    assert_eq!(
        startup.snapshot.settings().org.startup_folded,
        StartupVisibility::Showeverything
    );
    assert_eq!(startup.snapshot.origin(&["org", "startup_folded"]), None);
    assert_eq!(std::fs::read_to_string(root).unwrap(), "");
}

// Invalid values reject the full graph, even if a valid root would mask them.
// This guards normal diagnostics and prevents partial preference admission.
#[test]
fn policy_admission() {
    let f = Fixture::new();
    for (name, value) in [
        ("unknown", "'fold'"),
        ("case", "'SHOWALL'"),
        ("integer", "1"),
        ("boolean", "false"),
        ("array", "[]"),
        ("table", "{}"),
    ] {
        f.write(
            "base.toml",
            &format!("org.startup_folded={value}\neditor.tab_width=3"),
        );
        let root = f.write(
            "config.toml",
            "include=['base.toml']\norg.startup_folded='showall'",
        );
        let startup = prepare(&root);
        let diagnostic = startup.diagnostic.expect(name);
        assert_eq!(diagnostic.key, ["org", "startup_folded"], "{name}");
        assert_eq!(startup.snapshot.settings(), &Settings::default(), "{name}");
    }
}
