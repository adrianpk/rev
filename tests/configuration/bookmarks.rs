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
