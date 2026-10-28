use rev::config::{Rgb, Settings, prepare};
use rev::org::Role;

use crate::fixture::Fixture;

// Independent defaults pin every activated role. Included siblings must survive
// each root override, with provenance resolving to the leaf's actual source.
#[test]
fn palette_leaves() {
    let fixture = Fixture::new();
    let cases = [
        ("heading", Role::Heading, "#7aa2f7"),
        ("keyword", Role::Keyword, "#9d7cd8"),
        ("drawer", Role::Drawer, "#2ac3de"),
        ("property", Role::Property, "#2ac3de"),
        ("planning", Role::Planning, "#ff9e64"),
        ("block", Role::Block, "#9d7cd8"),
        ("code", Role::Code, "#9ece6a"),
        ("comment", Role::Comment, "#565f89"),
        ("list", Role::List, "#89ddff"),
        ("table", Role::Table, "#89ddff"),
        ("formula", Role::Formula, "#ff9e64"),
        ("emphasis", Role::Emphasis, "#9d7cd8"),
        ("link", Role::Link, "#7aa2f7"),
        ("target", Role::Target, "#2ac3de"),
        ("footnote", Role::Footnote, "#ff9e64"),
        ("entity", Role::Entity, "#2ac3de"),
        ("malformed", Role::Malformed, "#e0af68"),
    ];
    assert_eq!(cases.map(|(_, role, _)| role), Role::ALL);
    let base = cases
        .iter()
        .map(|(key, _, _)| format!("palette.org.{key} = '#102030'\n"))
        .collect::<String>();
    let included = fixture.write("base.toml", &base);
    for (key, role, default) in cases {
        assert_eq!(
            role.color(&Settings::default().palette.org),
            Rgb::parse(default).unwrap(),
            "{key}"
        );
        let root = fixture.write(
            "config.toml",
            &format!("include = ['base.toml']\npalette.org.{key} = '#012345'"),
        );
        let startup = prepare(&root);
        assert!(
            startup.diagnostic.is_none(),
            "{key}: {:?}",
            startup.diagnostic
        );
        assert_eq!(
            role.color(&startup.snapshot.settings().palette.org),
            Rgb(1, 35, 69),
            "{key}"
        );
        assert_eq!(
            startup.snapshot.origin(&["palette", "org", key]),
            Some(root.as_path())
        );
        inherited(&startup.snapshot, role, &included);
    }
}

fn inherited(snapshot: &rev::config::Snapshot, changed: Role, source: &std::path::Path) {
    let keys = [
        "heading",
        "keyword",
        "drawer",
        "property",
        "planning",
        "block",
        "code",
        "comment",
        "list",
        "table",
        "formula",
        "emphasis",
        "link",
        "target",
        "footnote",
        "entity",
        "malformed",
    ];
    for (key, role) in keys
        .into_iter()
        .zip(Role::ALL)
        .filter(|(_, role)| *role != changed)
    {
        assert_eq!(
            role.color(&snapshot.settings().palette.org),
            Rgb(16, 32, 48),
            "{key}"
        );
        assert_eq!(snapshot.origin(&["palette", "org", key]), Some(source));
    }
    assert_eq!(
        snapshot.settings().palette.syntax,
        Settings::default().palette.syntax
    );
}

// Invalid RGB and unknown Org keys reject the complete candidate with a source
// diagnostic, preserving the same whole-startup fallback as existing roles.
#[test]
fn rejected_colors() {
    let fixture = Fixture::new();
    for (name, text) in [
        ("short", "palette.org.heading = '#123'"),
        ("hex", "palette.org.code = '#12345g'"),
        ("wrong type", "palette.org.table = 3"),
        ("unknown", "palette.org.typo = '#123456'"),
    ] {
        let startup = prepare(&fixture.write("config.toml", text));
        assert!(startup.diagnostic.is_some(), "{name}");
        assert_eq!(startup.snapshot.settings(), &Settings::default(), "{name}");
    }
}
