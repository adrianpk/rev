use std::ffi::OsString;
use std::io::ErrorKind;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;

use rev::file::completion::{Provider, Query};

use super::fixture::Fixture;

// Parsing is lexical and IO-free, with caller-owned roots. Trailing slashes,
// dot parents, spaces and Unicode preserve the requested directory component.
#[test]
fn literal_queries() {
    let base = Path::new("/fixture/base");
    let home = Path::new("/fixture/home");
    for (draft, directory, prefix) in [
        ("", "/fixture/base", ""),
        ("alpha", "/fixture/base", "alpha"),
        ("./a", "/fixture/base/./", "a"),
        ("../a", "/fixture/base/../", "a"),
        ("sub/", "/fixture/base/sub/", ""),
        ("sub/../a", "/fixture/base/sub/../", "a"),
        ("/", "/", ""),
        ("/outside/a", "/outside/", "a"),
        ("~/", "/fixture/home/", ""),
        ("~//a", "/fixture/home/", "a"),
        ("~/界 dir/e\u{301}", "/fixture/home/界 dir/", "e\u{301}"),
        (" a dir/é", "/fixture/base/ a dir/", "é"),
        ("$HOME/*.rs", "/fixture/base/$HOME/", "*.rs"),
        ("$(touch marker)", "/fixture/base", "$(touch marker)"),
        ("~guest/a", "/fixture/base/~guest/", "a"),
        ("~", "/fixture/base", "~"),
    ] {
        let query = Query::parse(base, Some(home), draft).unwrap();
        assert_eq!(query.directory(), Path::new(directory), "{draft}");
        assert_eq!(query.prefix(), prefix, "{draft}");
    }
}

// Missing/relative roots and NUL cannot become implicit cwd/home fallbacks.
// An unavailable home affects only the explicitly requested ~/ expansion.
#[test]
fn rejected_queries() {
    for (base, home, draft) in [
        ("relative", None, "a"),
        ("/base", None, "~/a"),
        ("/base", Some("relative"), "~/a"),
        ("/base", None, "bad\0name"),
        ("/base", None, "bad\0parent/name"),
    ] {
        let error = Query::parse(Path::new(base), home.map(Path::new), draft).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput, "{draft:?}");
    }
    assert_eq!(
        Query::parse(Path::new("/base"), None, "literal")
            .unwrap()
            .prefix(),
        "literal"
    );
    assert_eq!(
        Provider::observe(Path::new("relative")).unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
}

// Display labels escape every terminal control without changing the selected
// path. Spaces, combining text and wide scalars stay exact identities and labels.
#[test]
fn safe_identities() {
    let fixture = Fixture::new();
    for name in [
        "wide 界e\u{301}",
        "line\nname",
        "tab\tname",
        "escape\u{1b}[31m",
        "c1\u{85}name",
    ] {
        fixture.write(name, "retained");
    }
    let provider = Provider::observe(&fixture.0).unwrap();
    for entry in provider.entries() {
        assert!(!entry.label().chars().any(char::is_control));
        assert_eq!(entry.path().unwrap(), fixture.0.join(entry.name().unwrap()));
        assert_eq!(std::fs::read(entry.path().unwrap()).unwrap(), b"retained");
    }
    let unicode = provider.matching("wide")[0];
    assert_eq!(unicode.label(), "wide 界e\u{301}");
    let escaped = provider.matching("escape")[0];
    assert_eq!(escaped.label(), "escape\\u{1b}[31m");
    assert_ne!(escaped.path().unwrap(), fixture.0.join(escaped.label()));
}

// Unsupported names remain visible as explicit diagnostics. They retain raw
// matching identity but cannot expose an opening path through a replacement name.
#[test]
fn unsupported_names() {
    let fixture = Fixture::new();
    let raw = OsString::from_vec(b"bad-\xff\x1bname".to_vec());
    std::fs::write(fixture.0.join(&raw), "retained").unwrap();
    let provider = Provider::observe(&fixture.0).unwrap();
    let matches = provider.matching("bad-");
    assert_eq!(matches.len(), 1);
    let entry = matches[0];
    assert!(entry.name().is_none());
    assert!(entry.label().contains("unsupported non-UTF-8 filename"));
    assert!(!entry.label().chars().any(char::is_control));
    assert_eq!(entry.path().unwrap_err().kind(), ErrorKind::InvalidData);
    assert_eq!(std::fs::read(fixture.0.join(raw)).unwrap(), b"retained");
}
