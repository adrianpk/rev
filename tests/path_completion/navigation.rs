use rev::file::completion::Navigation;

use super::fixture::Fixture;

// Tab fills the default first match of an ambiguous query or its explicitly
// selected alternative. Repeated completion never cycles or dispatches a file.
#[test]
fn completion_rules() {
    let fixture = Fixture::new();
    for name in ["alpha.txt", "alpine.txt", "solo.txt"] {
        fixture.write(name, "retained");
    }
    let mut navigation = Navigation::new(fixture.0.clone(), None);
    navigation.update("al", false);
    assert_eq!(navigation.complete().unwrap().as_deref(), Some("alpha.txt"));
    navigation.update("alp", false);
    assert_eq!(navigation.complete().unwrap().as_deref(), Some("alpha.txt"));
    navigation.navigate(false);
    assert_eq!(
        navigation.complete().unwrap().as_deref(),
        Some("alpine.txt")
    );
    navigation.update("sol", false);
    assert_eq!(navigation.complete().unwrap().as_deref(), Some("solo.txt"));
}

// A completed directory keeps its separator and observes that component. Editing
// back to the parent invalidates the child snapshot and restores parent matches.
#[test]
fn directory_transitions() {
    let fixture = Fixture::new();
    fixture.write("child/nested.txt", "retained");
    let mut navigation = Navigation::new(fixture.0.clone(), None);
    navigation.update("ch", false);
    assert_eq!(navigation.complete().unwrap().as_deref(), Some("child/"));
    navigation.update("child/", false);
    assert_eq!(navigation.directory().unwrap(), fixture.0.join("child"));
    assert_eq!(navigation.candidates()[0].name(), Some("nested.txt"));
    assert_eq!(
        navigation.complete().unwrap().as_deref(),
        Some("child/nested.txt")
    );
    navigation.update("child", false);
    assert_eq!(navigation.directory().unwrap(), fixture.0);
    assert_eq!(navigation.candidates()[0].label(), "child/");
    navigation.update("child/../", false);
    assert_eq!(navigation.candidates()[0].label(), "child/");
}

// Full selected names retain whole combining/ZWJ clusters. Completion inserts
// exact filename bytes, even when the query stops within a candidate cluster.
#[test]
fn grapheme_names() {
    for (left, right, query, expected) in [
        ("界e\u{301}a", "界e\u{301}b", "界", Some("界e\u{301}a")),
        ("e\u{301}a", "e\u{300}b", "e", Some("e\u{300}b")),
        ("👩‍💻a", "👩‍💻b", "👩", Some("👩‍💻a")),
    ] {
        let fixture = Fixture::new();
        fixture.write(left, "retained");
        fixture.write(right, "retained");
        let mut navigation = Navigation::new(fixture.0.clone(), None);
        navigation.update(query, false);
        assert_eq!(
            navigation.complete().unwrap().as_deref(),
            expected,
            "{query}"
        );
    }
}

// Refresh tracks the selected identity when sorted rows shift. A removed choice
// reports an error instead of completing the unrelated row at its old index.
#[test]
fn refreshed_selection() {
    let fixture = Fixture::new();
    fixture.write("bravo", "retained");
    let selected = fixture.write("charlie", "retained");
    let mut navigation = Navigation::new(fixture.0.clone(), None);
    navigation.navigate(false);
    fixture.write("alpha", "retained");
    assert_eq!(navigation.complete().unwrap().as_deref(), Some("charlie"));
    assert_eq!(navigation.selected(), Some(2));
    std::fs::remove_file(selected).unwrap();
    assert!(
        navigation
            .complete()
            .unwrap_err()
            .to_string()
            .contains("no longer present")
    );
}

// A changed prefix reuses the snapshot; explicit completion sees additions and
// reports a disappeared directory without replacing the editable draft.
#[test]
fn explicit_observation() {
    let fixture = Fixture::new();
    fixture.write("root/first", "retained");
    let base = fixture.0.join("root");
    let mut navigation = Navigation::new(base.clone(), None);
    fixture.write("root/added", "retained");
    navigation.update("ad", false);
    assert!(navigation.candidates().is_empty());
    assert_eq!(navigation.complete().unwrap().as_deref(), Some("added"));
    std::fs::rename(&base, fixture.0.join("moved")).unwrap();
    navigation.update("add", false);
    assert_eq!(navigation.candidates()[0].name(), Some("added"));
    assert!(navigation.complete().is_err());
    assert!(navigation.diagnostic().unwrap().contains("root"));
    assert_eq!(navigation.typed_path().unwrap(), base.join("add"));
}
