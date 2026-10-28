use super::fixture::Fixture;
use rev::{
    config::{Snapshot, prepare},
    org::agenda::SourceSelection,
};
// Sources replace by leaf, retain their declaring source, and resolve only
// when explicitly requested. Invalid limits reject the whole candidate.
#[test]
fn agenda_configuration() {
    let f = Fixture::new();
    let base = f.write(
        "settings/base.toml",
        "org.agenda.sources=['tasks.org']\norg.limits.agenda_files=8",
    );
    let root = f.write(
        "config.toml",
        "include=['settings/base.toml']\norg.limits.agenda_file_bytes=42",
    );
    let s = prepare(&root);
    assert!(s.diagnostic.is_none(), "{:?}", s.diagnostic);
    assert_eq!(
        SourceSelection::configured(&s.snapshot).unwrap().paths,
        vec![f.0.join("settings/tasks.org")]
    );
    assert_eq!(
        s.snapshot.origin(&["org", "agenda", "sources"]),
        Some(base.as_path())
    );
    assert_eq!(s.snapshot.settings().org.limits.agenda_files, 8);
    assert_eq!(s.snapshot.settings().org.limits.agenda_file_bytes, 42);
    let root = f.write(
        "config.toml",
        "include=['settings/base.toml']\norg.agenda.sources=[]",
    );
    assert!(
        SourceSelection::configured(&prepare(&root).snapshot)
            .unwrap()
            .paths
            .is_empty()
    );
    assert!(!f.0.join("settings/tasks.org").exists());
    for (name, value) in [
        ("zero", "org.limits.agenda_files=0"),
        ("files", "org.limits.agenda_files=4097"),
        ("file-bytes", "org.limits.agenda_file_bytes=33554433"),
        ("total", "org.limits.agenda_total_bytes=268435457"),
        ("time", "org.limits.job_seconds=601"),
        ("path", "org.agenda.sources=['']"),
        (
            "url",
            "org.agenda.sources=['https://example.invalid/tasks.org']",
        ),
    ] {
        let p = f.write("candidate.toml", value);
        let s = prepare(&p);
        assert!(s.diagnostic.is_some(), "{name}");
        assert_eq!(s.snapshot, Snapshot::default(), "{name}");
    }
    for (name, source) in [
        (
            "roots",
            format!(
                "org.agenda.sources=[{}]",
                vec!["'task.org'"; 4097].join(",")
            ),
        ),
        (
            "path-bytes",
            format!("org.agenda.sources=['{}']", "a".repeat(4097)),
        ),
        ("negative", "org.limits.job_seconds=-1".into()),
    ] {
        let path = f.write("candidate.toml", &source);
        let s = prepare(&path);
        assert!(s.diagnostic.is_some(), "{name}");
        assert_eq!(s.snapshot, Snapshot::default(), "{name}");
    }
}
