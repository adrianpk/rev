use rev::config::{
    Document, Roots, Settings, Snapshot, default_template, load, parse, prepare, resolve,
};

use crate::fixture::Fixture;

// A malformed root, a late invalid include and a missing explicit include each
// select complete defaults, withholding every otherwise valid candidate leaf.
#[test]
fn whole_candidate_fallback() {
    let cases = [
        (
            "root",
            "include = ['valid.toml']\neditor.tab_width = 8\npalette.background = '#bad'",
            "config.toml",
        ),
        (
            "included value",
            "include = ['valid.toml', 'invalid.toml']\neditor.tab_width = 8",
            "invalid.toml",
        ),
        (
            "missing include",
            "include = ['valid.toml', 'missing.toml']\neditor.tab_width = 8",
            "config.toml",
        ),
    ];
    for (name, text, failed_source) in cases {
        let fixture = Fixture::new();
        let root = fixture.write("config.toml", text);
        fixture.write(
            "valid.toml",
            "syntax.enabled = false\nlanguage.go.indent_width = 2\npalette.foreground = '#ffffff'",
        );
        fixture.write("invalid.toml", "language.go.indent_width = 0");
        let startup = prepare(&root);
        assert_eq!(startup.snapshot, Snapshot::default(), "{name}");
        assert_eq!(startup.snapshot.settings(), &Settings::default(), "{name}");
        let diagnostic = startup.diagnostic.unwrap();
        assert_eq!(diagnostic.source, fixture.0.join(failed_source), "{name}");
        assert!(!diagnostic.message.is_empty(), "{name}");
        assert_eq!(
            startup.snapshot.origin(&["syntax", "enabled"]),
            None,
            "{name}"
        );
    }
}

// A missing optional root is the successful built-in candidate, with no error
// and no filesystem creation, rather than a rejected explicit source. Packaged
// access, parsing, path resolution, loading and projection remain pure helpers.
#[test]
fn missing_candidate() {
    let fixture = Fixture::new();
    let root = fixture.0.join("settings/config.toml");
    let roots =
        Roots::resolve(|name| (name == "HOME").then(|| fixture.0.join("home").into_os_string()))
            .unwrap();
    let document = parse(
        std::path::Path::new("packaged/config.toml"),
        default_template().as_bytes(),
    )
    .unwrap();
    assert_eq!(document.include, Some(Vec::new()));
    let sources = load(&root).unwrap();
    assert!(sources.sources.is_empty());
    assert_eq!(resolve(&sources).unwrap(), Snapshot::default());
    let startup = prepare(&root);
    assert_eq!(startup.snapshot, Snapshot::default());
    assert!(startup.diagnostic.is_none());
    assert!(!root.parent().unwrap().exists());
    for path in [roots.config, roots.data, roots.state, roots.cache] {
        assert!(!path.exists());
    }
    assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 0);
}

// Sources are public data, so projection revalidates them. An invalid lower
// layer must not hide behind a valid overriding root and reach unchecked casts.
#[test]
fn validate_before_projection() {
    let fixture = Fixture::new();
    let root = fixture.write(
        "config.toml",
        "include = ['base.toml']\neditor.indent_width = 4\npalette.background = '#000000'",
    );
    let included = fixture.write("base.toml", "editor.indent_width = 2");
    for name in ["integer", "color", "command"] {
        let mut sources = load(&root).unwrap();
        corrupt(&mut sources.sources[1].document, name);
        let error = resolve(&sources).unwrap_err();
        assert_eq!(error.source, included, "{name}");
        assert!(!error.key.is_empty(), "{name}");
    }
}

fn corrupt(document: &mut Document, name: &str) {
    match name {
        "integer" => document.editor.indent_width = Some(-1),
        "color" => document.palette.background = Some("#invalid".into()),
        "command" => {
            document
                .keys
                .editor
                .insert("M-j".into(), "run-shell".into());
        }
        _ => unreachable!("known invalid candidate"),
    }
}

// Snapshot contents remain owned and fixed after preparation; a later explicit
// preparation observes file changes without altering the earlier snapshot.
#[test]
fn fixed_snapshot() {
    let fixture = Fixture::new();
    let root = fixture.write(
        "config.toml",
        "editor.tab_width = 8\nlanguage.go.indent_width = 2",
    );
    let first = prepare(&root);
    assert!(first.diagnostic.is_none());
    fixture.write(
        "config.toml",
        "editor.tab_width = 3\nlanguage.go.indent_width = 6",
    );
    let next = prepare(&root);
    assert!(next.diagnostic.is_none());
    assert_eq!(
        (
            first.snapshot.editor("go").indent_width,
            first.snapshot.editor("go").tab_width
        ),
        (2, 8)
    );
    assert_eq!(
        (
            next.snapshot.editor("go").indent_width,
            next.snapshot.editor("go").tab_width
        ),
        (6, 3)
    );
}

// Incomplete, duplicate or invalid occurrence indices are rejected before
// indexing public source records or publishing a misleading partial snapshot.
#[test]
fn invalid_source_order() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "include = ['base.toml']");
    fixture.write("base.toml", "editor.tab_width = 8");
    for (name, order) in [
        ("incomplete", vec![0]),
        ("duplicate", vec![0, 0]),
        ("outside", vec![0, 3]),
    ] {
        let mut sources = load(&root).unwrap();
        sources.order = order;
        let error = resolve(&sources).unwrap_err();
        assert!(error.message.contains("source occurrence order"), "{name}");
    }
}
