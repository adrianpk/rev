use super::*;
use rev::org::{
    Cancellation,
    agenda::{
        AgendaQuery, CalendarQuery, QueryKind as K, QueryResult, SourceScan, SourceSelection,
        SourceVersion,
    },
};
fn query(config: &str, source: &str, kind: K, expression: &str) -> (Fixture, QueryResult) {
    let f = Fixture::new();
    let config = prepare(&f.write("config.toml", config));
    assert!(config.diagnostic.is_none(), "{:?}", config.diagnostic);
    let policy = config.snapshot.settings().org.clone();
    let mut buffers = Buffers::with_settings(config.snapshot).unwrap();
    let path = f.write("tasks.org", source);
    buffers.visit(&path).unwrap();
    let sources = SourceScan::new(
        &buffers,
        SourceSelection {
            paths: vec![path],
            buffers: Vec::new(),
        },
        &policy.limits,
    )
    .unwrap()
    .run(&Cancellation::default());
    let query = AgendaQuery::parse(kind, expression, &policy.limits).unwrap();
    let context = CalendarQuery::new(super::habits::clock(&policy.time).unwrap(), &policy).unwrap();
    let result =
        QueryResult::observe(&sources, query, context, &policy, &Cancellation::default()).unwrap();
    (f, result)
}
const SOURCE: &str = "#+TODO: TODO NEXT | CLOSED\n#+FILETAGS: :global:\n#+PRIORITIES: A D C\n* TODO [#A] Parent :work:\n:PROPERTIES:\n:TEAM: red\n:EFFORT: 1:30\n:boss-prio: C\n:END:\nComputer\n equipment 8.11b\n** NEXT Child :urgent:\nChild only\n* CLOSED Finished :work:\n* Note\nWords only\n* COMMENT Hidden\nComputer equipment\n** NEXT Hidden child\n* Archived :ARCHIVE:\nComputer equipment\n";
// Metadata queries use real structural/file declarations, local workflows,
// selective inherited properties and native effort without changing source.
#[test]
fn query_canonical_metadata() {
    let config = "org.properties.inherit=['TEAM','EFFORT']\norg.time.zone='UTC'";
    for (kind, expression, expected) in [
        (K::Todo, "", vec!["Parent", "Child"]),
        (K::Todo, "CLOSED", vec!["Finished"]),
        (
            K::Tags,
            r#"work+urgent+TEAM="red"+EFFORT<*120"#,
            vec!["Child"],
        ),
        (K::Tags, "global/!NEXT", vec!["Child"]),
        (K::Tags, r#"LEVEL=2+PRIORITY="C""#, vec!["Child"]),
        (K::Tags, r#"boss\-prio="C""#, vec!["Parent"]),
        (K::Search, "computer equipment", vec!["Parent"]),
        (
            K::Search,
            r"+computer +{8\.11[bg]} -ethernet",
            vec!["Parent"],
        ),
    ] {
        let (f, result) = query(config, SOURCE, kind, expression);
        assert!(result.complete(), "{expression}: {:?}", result.issues);
        assert_eq!(
            result
                .rows
                .iter()
                .map(|r| r.title.as_ref())
                .collect::<Vec<_>>(),
            expected,
            "{expression}"
        );
        for row in &result.rows {
            assert!(SOURCE[row.byte..].starts_with('*'));
            assert!(matches!(
                result.sources[row.source].version,
                SourceVersion::Buffer(_)
            ));
        }
        assert_eq!(
            std::fs::read_to_string(f.0.join("tasks.org")).unwrap(),
            SOURCE
        );
    }
    let (_, result) = query("", SOURCE, K::Tags, "work+urgent+EFFORT<*120");
    assert!(result.rows.is_empty());
    assert!(result.complete());
}
// Exact row admission is complete; excess or malformed canonical metadata
// remains explicitly incomplete with real source identity and diagnostics.
#[test]
fn query_partial_results() {
    for (config, source, expression, count, complete) in [
        ("org.limits.agenda_rows=1", "* TODO One", "", 1, true),
        (
            "org.limits.agenda_rows=1",
            "* TODO One\n* TODO Two",
            "",
            1,
            false,
        ),
    ] {
        let (_, result) = query(config, source, K::Todo, expression);
        assert_eq!(result.rows.len(), count);
        assert_eq!(result.complete(), complete, "{:?}", result.issues);
    }
    let (_, result) = query(
        "",
        "* TODO One\n:PROPERTIES:\n:TEAM: a\n:TEAM: b\n:END:",
        K::Tags,
        r#"TEAM="a""#,
    );
    assert!(!result.complete());
    assert!(!result.issues.is_empty());
}
