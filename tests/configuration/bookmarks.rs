use super::fixture::Fixture;
use rev::config::{BookmarkSave, Document, Settings, parse, prepare};
use std::path::Path;

// Defaults, explicit zero, absent siblings and include provenance use the
// existing configuration pipeline without rewriting the user's file.
#[test]
fn storage_settings() {
    let defaults = Settings::default().bookmarks;
    assert_eq!(defaults.file, None);
    assert_eq!(defaults.save, BookmarkSave::OnExit);
    assert!(defaults.markers);
    assert_eq!(defaults.marker, "●");
    assert_eq!(
        (
            defaults.max_entries,
            defaults.context_chars,
            defaults.search_chars
        ),
        (1024, 64, 262144)
    );
    let f = Fixture::new();
    f.write(
        "base.toml",
        "[bookmarks]\nmax_entries=2\ncontext_chars=16\nsave='manual'\n",
    );
    let text = format!(
        "include=['base.toml']\n[bookmarks]\nfile='{}'\nsearch_chars=0\n",
        f.0.join("state.toml").display()
    );
    let root = f.write("config.toml", &text);
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let settings = &startup.snapshot.settings().bookmarks;
    assert_eq!(
        (
            settings.max_entries,
            settings.context_chars,
            settings.search_chars
        ),
        (2, 16, 0)
    );
    assert_eq!(settings.save, BookmarkSave::Manual);
    assert_eq!(
        startup.snapshot.origin(&["bookmarks", "save"]),
        Some(f.0.join("base.toml").as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["bookmarks", "search_chars"]),
        Some(root.as_path())
    );
    assert_eq!(std::fs::read_to_string(root).unwrap(), text);
    let later: Document = toml::from_str("[bookmarks]\nsearch_chars=0").unwrap();
    assert_eq!(later.bookmarks.max_entries, None);
    assert_eq!(later.bookmarks.search_chars, Some(0));
}

// Marker glyphs are checked as complete Unicode graphemes and terminal cells,
// with whole-candidate rejection and independent include provenance for leaves.
#[test]
fn marker_settings() {
    for (name, marker, accepted) in [
        ("factory", "●", true),
        ("ascii", "*", true),
        ("combined", "e\u{301}", true),
        ("empty", "", false),
        ("several", "ab", false),
        ("wide", "界", false),
        ("emoji", "👩‍💻", false),
        ("orphan", "\u{301}", false),
        ("nul", "\0", false),
        ("escape", "\x1b", false),
        ("tab", "\t", false),
        ("newline", "\n", false),
    ] {
        let literal = marker
            .chars()
            .map(|ch| match ch {
                '\0' => "\\u0000".into(),
                '\x1b' => "\\u001b".into(),
                '\t' => "\\t".into(),
                '\n' => "\\n".into(),
                ch => ch.to_string(),
            })
            .collect::<String>();
        let result = parse(
            Path::new("markers.toml"),
            format!("bookmarks.marker=\"{literal}\"").as_bytes(),
        );
        assert_eq!(result.is_ok(), accepted, "{name}");
    }
    let f = Fixture::new();
    let base = f.write(
        "base.toml",
        "[bookmarks]\nmarkers=false\nmarker='*'\n[palette.gutter]\nbookmark='#123456'\n",
    );
    let root = f.write(
        "config.toml",
        "include=['base.toml']\n[bookmarks]\nmarker='é'\n",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert!(!startup.snapshot.settings().bookmarks.markers);
    assert_eq!(startup.snapshot.settings().bookmarks.marker, "é");
    assert_eq!(
        startup.snapshot.settings().palette.gutter.bookmark,
        rev::config::Rgb(0x12, 0x34, 0x56)
    );
    assert_eq!(
        startup.snapshot.origin(&["bookmarks", "markers"]),
        Some(base.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["bookmarks", "marker"]),
        Some(root.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["palette", "gutter", "bookmark"]),
        Some(base.as_path())
    );
    f.write("config.toml", "bookmarks.marker='界'\neditor.tab_width=9\n");
    let rejected = prepare(&root);
    assert!(rejected.diagnostic.is_some());
    assert_eq!(rejected.snapshot.settings(), &Settings::default());
    for value in ["'bad'", "true", "'#12345g'"] {
        assert!(
            parse(
                Path::new("config.toml"),
                format!("palette.gutter.bookmark={value}").as_bytes()
            )
            .is_err()
        );
    }
}

// Reject invalid values at the source boundary rather than clamping or
// activating a partial candidate. Published numeric limits include endpoints.
#[test]
fn storage_bounds() {
    for (key, values) in [
        ("max_entries", vec!["0", "4097", "-1"]),
        ("context_chars", vec!["15", "257", "-1"]),
        ("search_chars", vec!["-1", "1048577"]),
        ("save", vec!["'sometimes'", "1"]),
        (
            "file",
            vec!["''", "'relative'", "'~/bookmarks'", "\"/bad\\u0000path\""],
        ),
    ] {
        for value in values {
            assert!(
                parse(
                    Path::new("config.toml"),
                    format!("bookmarks.{key}={value}").as_bytes()
                )
                .is_err(),
                "{key}={value}"
            );
        }
    }
    for (key, values) in [
        ("max_entries", [1, 4096]),
        ("context_chars", [16, 256]),
        ("search_chars", [0, 1048576]),
    ] {
        for value in values {
            assert!(
                parse(
                    Path::new("config.toml"),
                    format!("bookmarks.{key}={value}").as_bytes()
                )
                .is_ok(),
                "{key}={value}"
            );
        }
    }
}
