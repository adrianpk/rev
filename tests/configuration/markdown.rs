use rev::config::{MarkdownPalette, Rgb, Settings, prepare};

use super::fixture::Fixture;

fn colors(palette: &MarkdownPalette) -> [Rgb; 12] {
    [
        palette.heading,
        palette.heading_2,
        palette.heading_3,
        palette.heading_4,
        palette.heading_5,
        palette.heading_6,
        palette.emphasis,
        palette.link,
        palette.code,
        palette.quote,
        palette.list,
        palette.table,
    ]
}

const KEYS: [&str; 12] = [
    "heading",
    "heading_2",
    "heading_3",
    "heading_4",
    "heading_5",
    "heading_6",
    "emphasis",
    "link",
    "code",
    "quote",
    "list",
    "table",
];

// Each leaf must merge independently, retain its include/root provenance and
// leave unrelated Org/syntax colors unchanged, including equal factory RGBs.
#[test]
fn leaf_inheritance() {
    let fixture = Fixture::new();
    let defaults = Settings::default();
    assert!(defaults.markdown.enabled);
    assert!(defaults.markdown.pretty);
    assert_eq!(
        colors(&defaults.palette.markdown),
        [
            Rgb(122, 162, 247),
            Rgb(157, 124, 216),
            Rgb(192, 202, 245),
            Rgb(155, 185, 249),
            Rgb(181, 156, 225),
            Rgb(188, 208, 251),
            Rgb(157, 124, 216),
            Rgb(122, 162, 247),
            Rgb(158, 206, 106),
            Rgb(86, 95, 137),
            Rgb(137, 221, 255),
            Rgb(137, 221, 255),
        ]
    );
    let base = KEYS
        .iter()
        .map(|key| format!("palette.markdown.{key}='#102030'\n"))
        .collect::<String>();
    let included = fixture.write(
        "base.toml",
        &format!("markdown.enabled=false\nmarkdown.pretty=false\n{base}"),
    );
    for (index, key) in KEYS.iter().enumerate() {
        let path = fixture.write(
            "config.toml",
            &format!(
                "include=['base.toml']\nmarkdown.pretty=true\nmarkdown.enabled=true\npalette.markdown.{key}='#012345'"
            ),
        );
        let startup = prepare(&path);
        assert!(
            startup.diagnostic.is_none(),
            "{key}: {:?}",
            startup.diagnostic
        );
        assert!(startup.snapshot.settings().markdown.enabled);
        assert!(startup.snapshot.settings().markdown.pretty);
        assert_eq!(
            startup.snapshot.origin(&["markdown", "pretty"]),
            Some(path.as_path())
        );
        assert_eq!(
            startup.snapshot.origin(&["markdown", "enabled"]),
            Some(path.as_path())
        );
        assert_eq!(
            startup.snapshot.settings().palette.org,
            defaults.palette.org
        );
        assert_eq!(
            startup.snapshot.settings().palette.syntax,
            defaults.palette.syntax
        );
        check_siblings(&startup.snapshot, index, &path, &included);
    }
}

fn check_siblings(
    snapshot: &rev::config::Snapshot,
    changed: usize,
    root: &std::path::Path,
    included: &std::path::Path,
) {
    for (index, (key, color)) in KEYS
        .iter()
        .zip(colors(&snapshot.settings().palette.markdown))
        .enumerate()
    {
        let (expected, origin) = if index == changed {
            (Rgb(1, 35, 69), root)
        } else {
            (Rgb(16, 32, 48), included)
        };
        assert_eq!(color, expected, "{key}");
        assert_eq!(snapshot.origin(&["palette", "markdown", key]), Some(origin));
    }
}

// Invalid RGB, type or unknown leaves reject the whole configuration rather
// than activating a partial palette or a disabled mode from a bad candidate.
#[test]
fn candidate_rejection() {
    let fixture = Fixture::new();
    for key in KEYS {
        for value in ["'#12345'", "'#gggggg'", "''", "false"] {
            let path = fixture.write(
                "bad.toml",
                &format!("markdown.enabled=false\npalette.markdown.{key}={value}"),
            );
            let startup = prepare(&path);
            assert!(startup.diagnostic.is_some(), "{key}={value}");
            assert_eq!(startup.snapshot.settings(), &Settings::default());
        }
    }
    for source in [
        "markdown.enabled=1",
        "markdown.pretty=1",
        "markdown.unknown=true",
        "palette.markdown.unknown='#012345'",
    ] {
        let startup = prepare(&fixture.write("bad.toml", source));
        assert!(startup.diagnostic.is_some(), "{source}");
        assert_eq!(startup.snapshot.settings(), &Settings::default());
    }
}

// An explicit false root overrides a true include, retaining its own provenance.
#[test]
fn source_preference() {
    let fixture = Fixture::new();
    fixture.write("base.toml", "markdown.pretty=true");
    let path = fixture.write(
        "config.toml",
        "include=['base.toml']\nmarkdown.pretty=false",
    );
    let startup = prepare(&path);
    assert!(startup.diagnostic.is_none());
    assert!(!startup.snapshot.settings().markdown.pretty);
    assert_eq!(
        startup.snapshot.origin(&["markdown", "pretty"]),
        Some(path.as_path())
    );
}

// Normalized local aliases retain include/root provenance and leave unrelated
// inherited editing routes intact. Factory Return cannot displace a user remap.
#[test]
fn editing_keymap() {
    use rev::command::Binding;
    use rev::input::keymap::{Context, Sequence};

    let fixture = Fixture::new();
    let base = fixture.write(
        "base.toml",
        "keys.markdown.'ESC j'='markdown-continue-item'\nkeys.markdown.'M-k'='markdown-outdent-item'",
    );
    let root = fixture.write(
        "config.toml",
        "include=['base.toml']\nkeys.editor.'C-m'='newline'\nkeys.markdown.'M-j'='markdown-indent-item'",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert_eq!(
        startup.snapshot.origin(&["keys", "markdown", "M-k"]),
        Some(base.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["keys", "markdown", "M-j"]),
        Some(root.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["keys", "markdown", "ESC j"]),
        None
    );
    let map = startup.snapshot.keymaps().map(Context::Markdown);
    for (key, action) in [
        ("M-j", "markdown-indent-item"),
        ("M-k", "markdown-outdent-item"),
        ("RET", "newline"),
        ("C-x C-s", "save-buffer"),
        ("M-RET", "markdown-meta-return"),
        ("M-S-Down", "markdown-table-insert-row"),
        ("M-S-Up", "markdown-table-delete-row"),
        ("M-S-Right", "markdown-table-insert-column"),
        ("M-S-Left", "markdown-table-delete-column"),
    ] {
        assert_eq!(
            map.lookup(&Sequence::parse(key).unwrap()),
            Binding::named(action),
            "{key}"
        );
    }
}

// Bad local values, terminal aliases, cancellation and unavailable prompt
// actions reject the whole candidate, including an otherwise valid mode change.
#[test]
fn editing_map_rejection() {
    let fixture = Fixture::new();
    for source in [
        "keys.markdown.RET=1",
        "keys.markdown.'C-g'='markdown-return'",
        "keys.markdown.RET='markdown-absent'",
        "keys.markdown.RET='newline'\nkeys.markdown.'C-m'='markdown-return'",
        "keys.markdown.'M-j'='minibuffer-accept'",
    ] {
        let path = fixture.write("bad.toml", &format!("markdown.enabled=false\n{source}"));
        let startup = prepare(&path);
        assert!(startup.diagnostic.is_some(), "{source}");
        assert_eq!(startup.snapshot.settings(), &Settings::default());
    }
}
