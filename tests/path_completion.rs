#![forbid(unsafe_code)]

#[path = "path_completion/acceptance.rs"]
mod acceptance;
#[path = "path_completion/continuation.rs"]
mod continuation;
#[path = "path_completion/edges.rs"]
mod edges;
#[path = "path_completion/failures.rs"]
mod failures;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "path_completion/identity.rs"]
mod identity;
#[path = "path_completion/navigation.rs"]
mod navigation;
#[path = "path_completion/observation.rs"]
mod observation;
#[path = "path_completion/panel.rs"]
mod panel;
#[path = "path_completion/stale.rs"]
mod stale;
#[path = "path_completion/visits.rs"]
mod visits;

use rev::file::completion::Provider;

// One directory is sorted by kind then filename bytes, including hidden names.
// Nested contents are not observed and presentation markers are not path bytes.
#[test]
fn ordered_component() {
    let fixture = fixture::Fixture::new();
    for name in [
        "z-dir/nested",
        "A-dir/nested",
        "z.txt",
        "a.txt",
        "A.txt",
        ".hidden",
        "界.txt",
        "é.txt",
    ] {
        fixture.write(name, "retained");
    }
    let provider = Provider::observe(&fixture.0).unwrap();
    let labels: Vec<_> = provider
        .entries()
        .iter()
        .map(|entry| entry.label())
        .collect();
    assert_eq!(
        labels,
        [
            "A-dir/", "z-dir/", ".hidden", "A.txt", "a.txt", "z.txt", "é.txt", "界.txt"
        ]
    );
    assert_eq!(
        provider.entries()[0].path().unwrap(),
        fixture.0.join("A-dir")
    );
    assert!(provider.diagnostic().is_none());
}

// Empty, shared, unique, case-sensitive and literal metacharacter prefixes use
// one cached observation; filtering cannot read, create or traverse anything.
#[test]
fn cached_prefixes() {
    let fixture = fixture::Fixture::new();
    for name in ["alpha", "alpine", "beta", "a*literal"] {
        fixture.write(&format!("source/{name}"), "retained");
    }
    let provider = Provider::observe(&fixture.0.join("source")).unwrap();
    std::fs::rename(fixture.0.join("source"), fixture.0.join("moved")).unwrap();
    for (prefix, expected) in [
        ("", vec!["a*literal", "alpha", "alpine", "beta"]),
        ("al", vec!["alpha", "alpine"]),
        ("alpha", vec!["alpha"]),
        ("AL", vec![]),
        ("a*", vec!["a*literal"]),
        ("none", vec![]),
    ] {
        let actual: Vec<_> = provider
            .matching(prefix)
            .iter()
            .map(|entry| entry.name().unwrap())
            .collect();
        assert_eq!(actual, expected, "{prefix}");
    }
}
