use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::{PermissionsExt, symlink};

use rev::file::completion::{ENTRY_LIMIT, INCOMPLETE_MESSAGE, NAME_BYTE_LIMIT, Provider, Query};

use super::fixture::Fixture;

// Limits apply before another entry is read. At the exact entry cap the provider
// conservatively reports incompleteness rather than peeking beyond its budget.
#[test]
fn entry_limits() {
    for (count, incomplete) in [
        (ENTRY_LIMIT - 1, false),
        (ENTRY_LIMIT, true),
        (ENTRY_LIMIT + 1, true),
    ] {
        let fixture = Fixture::new();
        for index in 0..count {
            fixture.write(&format!("e{index:04}"), "");
        }
        let provider = Provider::observe(&fixture.0).unwrap();
        assert_eq!(provider.entries().len(), count.min(ENTRY_LIMIT));
        assert_eq!(
            provider.diagnostic(),
            incomplete.then_some(INCOMPLETE_MESSAGE)
        );
        assert!(provider.name_bytes() <= NAME_BYTE_LIMIT);
    }
}

// Multibyte names exhaust the byte budget before the entry cap. The first name
// that cannot fit ends observation, without retaining an over-budget candidate.
#[test]
fn name_limit() {
    let fixture = Fixture::new();
    let suffix = "界".repeat(83);
    for index in 0..2100 {
        fixture.write(&format!("{index:04}{suffix}"), "");
    }
    let provider = Provider::observe(&fixture.0).unwrap();
    assert_eq!(provider.entries().len(), NAME_BYTE_LIMIT / 253);
    assert_eq!(provider.name_bytes(), (NAME_BYTE_LIMIT / 253) * 253);
    assert_eq!(provider.diagnostic(), Some(INCOMPLETE_MESSAGE));
    assert!(provider.entries().len() < ENTRY_LIMIT);
}

// Only explicit refresh observes additions/removals. Filtering and labels keep
// their original snapshot until the complete replacement is successfully read.
#[test]
fn explicit_refresh() {
    let fixture = Fixture::new();
    let mut provider = Provider::observe(&fixture.0).unwrap();
    assert!(provider.entries().is_empty());
    assert!(provider.diagnostic().is_none());
    fixture.write("new.txt", "retained");
    assert!(provider.matching("new").is_empty());
    provider.refresh().unwrap();
    assert_eq!(provider.matching("new")[0].label(), "new.txt");
    fs::remove_file(fixture.0.join("new.txt")).unwrap();
    assert_eq!(provider.matching("new")[0].label(), "new.txt");
    provider.refresh().unwrap();
    assert!(provider.entries().is_empty());
    assert_eq!(provider.name_bytes(), 0);
    assert!(provider.diagnostic().is_none());
}

// Permission, disappearance and replacement-by-file errors preserve both the
// previous snapshot and IO error kind. Restore owned permissions before cleanup.
#[test]
fn failed_refresh() {
    for (name, expected) in [
        ("denied", ErrorKind::PermissionDenied),
        ("removed", ErrorKind::NotFound),
        ("file", ErrorKind::NotADirectory),
    ] {
        let fixture = Fixture::new();
        fixture.write("component/alpha", "retained");
        let directory = fixture.0.join("component");
        let permissions = fs::metadata(&directory).unwrap().permissions();
        let mut provider = Provider::observe(&directory).unwrap();
        match name {
            "denied" => fs::set_permissions(&directory, fs::Permissions::from_mode(0o0)).unwrap(),
            "removed" => fs::remove_dir_all(&directory).unwrap(),
            _ => {
                fs::remove_dir_all(&directory).unwrap();
                fs::write(&directory, "replacement").unwrap();
            }
        }
        let result = provider.refresh();
        if name == "denied" {
            fs::set_permissions(&directory, permissions).unwrap();
        }
        let error = result.unwrap_err();
        assert_eq!(error.kind(), expected, "{name}: {error}");
        assert!(error.to_string().contains("component"), "{name}: {error}");
        assert_eq!(provider.directory(), directory);
        assert_eq!(provider.matching("a")[0].label(), "alpha");
        assert_eq!(
            provider.matching("a")[0].path().unwrap(),
            directory.join("alpha")
        );
    }
}

// Directory, broken and looping child symlinks are all retained leaves. Listing
// cannot inspect their destinations, recursively enumerate or fail on a cycle.
#[test]
fn symlink_leaves() {
    let fixture = Fixture::new();
    fixture.write("component/directory/nested.txt", "retained");
    fixture.write("elsewhere/hidden.txt", "retained");
    let directory = fixture.0.join("component");
    symlink("../elsewhere", directory.join("alias")).unwrap();
    symlink("missing", directory.join("broken")).unwrap();
    symlink("loop", directory.join("loop")).unwrap();
    let provider = Provider::observe(&directory).unwrap();
    let labels: Vec<_> = provider
        .entries()
        .iter()
        .map(|entry| entry.label())
        .collect();
    assert_eq!(labels, ["directory/", "alias", "broken", "loop"]);
    for name in ["alias", "broken", "loop"] {
        let entry = provider.matching(name)[0];
        assert!(entry.is_symlink());
        assert!(!entry.is_directory());
        assert_eq!(entry.path().unwrap(), directory.join(name));
    }
}

// Explicit parent components retain OS symlink semantics. Lexically collapsing
// alias/.. would enumerate the wrong directory and choose a different identity.
#[test]
fn symlink_parent() {
    let fixture = Fixture::new();
    fixture.write("target/inner/unused.txt", "retained");
    fixture.write("target/expected.txt", "target");
    fixture.write("base/expected.txt", "wrong");
    let base = fixture.0.join("base");
    symlink("../target/inner", base.join("alias")).unwrap();
    let query = Query::parse(&base, None, "alias/../ex").unwrap();
    let provider = Provider::observe(query.directory()).unwrap();
    let matches = provider.matching(query.prefix());
    assert_eq!(matches.len(), 1);
    let path = matches[0].path().unwrap();
    assert_eq!(path, base.join("alias/../expected.txt"));
    assert_eq!(fs::read(path).unwrap(), b"target");
}

// Parsed directory components drive real observations with explicit base/home
// roots, including Unicode and spaces; neither shell expansion nor cwd is used.
#[test]
fn query_observations() {
    let fixture = Fixture::new();
    fixture.write("base/sub/item", "retained");
    fixture.write("base/space dir/界 name", "retained");
    fixture.write("home/item", "retained");
    let base = fixture.0.join("base");
    let home = fixture.0.join("home");
    for (draft, name) in [
        ("sub/it".to_owned(), "item"),
        ("./sub/it".to_owned(), "item"),
        ("../home/it".to_owned(), "item"),
        ("~/it".to_owned(), "item"),
        (format!("{}/it", home.display()), "item"),
        ("space dir/界".to_owned(), "界 name"),
    ] {
        let query = Query::parse(&base, Some(&home), &draft).unwrap();
        let provider = Provider::observe(query.directory()).unwrap();
        let matches = provider.matching(query.prefix());
        assert_eq!(matches.len(), 1, "{draft}");
        assert_eq!(matches[0].name(), Some(name), "{draft}");
        assert_eq!(fs::read(matches[0].path().unwrap()).unwrap(), b"retained");
    }
}
