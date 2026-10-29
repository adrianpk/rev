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

// Calendar span/weekday/row policies layer by typed leaf and reject future or
// out-of-range choices without partially activating the candidate.
#[test]
fn calendar_configuration() {
    let f = Fixture::new();
    let base = f.write(
        "base.toml",
        "org.agenda.span='day'\norg.limits.agenda_rows=5",
    );
    let root = f.write(
        "calendar.toml",
        "include=['base.toml']\norg.agenda.week_start=7",
    );
    let s = prepare(&root);
    assert!(s.diagnostic.is_none());
    assert_eq!(
        s.snapshot.settings().org.agenda.span,
        rev::config::AgendaSpan::Day
    );
    assert_eq!(s.snapshot.settings().org.agenda.week_start, 7);
    assert_eq!(s.snapshot.settings().org.limits.agenda_rows, 5);
    assert_eq!(
        s.snapshot.origin(&["org", "agenda", "span"]),
        Some(base.as_path())
    );
    for value in [
        "org.agenda.span='invalid'",
        "org.agenda.week_start=0",
        "org.agenda.week_start=8",
        "org.limits.agenda_rows=0",
        "org.limits.agenda_rows=100001",
        "org.limits.query_bytes=0",
        "org.limits.query_bytes=65537",
        "org.limits.query_depth=0",
        "org.limits.query_depth=129",
    ] {
        let s = prepare(&f.write("candidate.toml", value));
        assert!(s.diagnostic.is_some(), "{value}");
        assert_eq!(s.snapshot, Snapshot::default());
    }
}

// Typed records replace as one leaf; unknown fields and combinations reject
// before startup can observe sources or publish part of a view definition.
#[test]
fn custom_configuration() {
    let f = Fixture::new();
    let valid = "org.agenda.views=[{key='w',title='Work',blocks=[{kind='todo',filter='work',columns=['ITEM','EFFORT'],sort=['effort-up'],span=14,row_limit=10}]}]";
    let base = f.write("base.toml", valid);
    let startup = prepare(&f.write(
        "custom.toml",
        "include=['base.toml']\norg.agenda.span='month'",
    ));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert_eq!(startup.snapshot.settings().org.agenda.views.len(), 1);
    assert_eq!(
        startup.snapshot.origin(&["org", "agenda", "views"]),
        Some(base.as_path())
    );
    let reset = prepare(&f.write("custom.toml", "include=['base.toml']\norg.agenda.views=[]"));
    assert!(reset.diagnostic.is_none());
    assert!(reset.snapshot.settings().org.agenda.views.is_empty());
    for (name, source) in [
        ("invalid-query", valid.replace("filter='work'","filter='EFFORT=>bad'")),
        ("query-budget", format!("org.limits.query_bytes=2\n{valid}")),
        ("too-many-views", format!("org.agenda.views=[{}]", (0..65).map(|i| format!("{{key='v{i}',title='View',blocks=[{{}}]}}")).collect::<Vec<_>>().join(","))),
        ("too-many-columns",valid.replace("columns=['ITEM','EFFORT']", &format!("columns=[{}]",vec!["'ITEM'";17].join(",")))),
        ("unknown-kind",valid.replace("kind='todo'","kind='eval'")),
        ("unknown-field",valid.replace("row_limit=10","callback='run'")),
        ("zero-span",valid.replace("span=14","span=0")),
        ("large-span",valid.replace("span=14","span=367")),
        ("bad-sort",valid.replace("effort-up","arbitrary")),
        ("bad-column",valid.replace("'ITEM'","'(eval)'")),
        ("zero-rows",valid.replace("row_limit=10","row_limit=0")),
        ("empty-blocks","org.agenda.views=[{key='w',title='Work',blocks=[]}]".into()),
        ("too-many-blocks",format!("org.agenda.views=[{{key='w',title='Work',blocks=[{}]}}]",vec!["{kind='todo'}";33].join(","))),
        ("duplicate", "org.agenda.views=[{key='w',title='One',blocks=[{}]},{key='w',title='Two',blocks=[{}]}]".into()),
    ] {
        let startup = prepare(&f.write("invalid.toml",&source));
        assert!(startup.diagnostic.is_some(),"{name}");
        assert_eq!(startup.snapshot,Snapshot::default(),"{name}");
    }
}
