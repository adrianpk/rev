use super::*;
use rev::{
    config::{AgendaSpan, AgendaView},
    org::{
        Cancellation,
        agenda::{CalendarQuery, CustomView, SourceScan, SourceSelection},
        time::Date,
    },
};
// Leap months, calendar years and custom counts use civil boundaries and
// checked shifts, independently of the machine's zone or current clock.
#[test]
fn custom_spans() {
    let policy = rev::config::Org::default();
    let query = CalendarQuery::new(super::habits::clock(&policy.time).unwrap(), &policy).unwrap();
    for (span, date, days, next) in [
        (
            AgendaSpan::Month,
            Date::new(2024, 2, 17).unwrap(),
            29,
            Date::new(2024, 3, 1).unwrap(),
        ),
        (
            AgendaSpan::Year,
            Date::new(2024, 5, 17).unwrap(),
            366,
            Date::new(2025, 1, 1).unwrap(),
        ),
        (
            AgendaSpan::Days(14),
            Date::new(2024, 2, 17).unwrap(),
            14,
            Date::new(2024, 3, 2).unwrap(),
        ),
    ] {
        let span = query.at(date, span).unwrap();
        assert_eq!(span.days(), days);
        assert_eq!(span.shifted(1).unwrap().start, next);
        assert_eq!(
            span.shifted(1).unwrap().shifted(-1).unwrap().start,
            span.start
        );
    }
    assert!(
        query
            .at(Date::new(9999, 12, 31).unwrap(), AgendaSpan::Days(2))
            .is_err()
    );
    assert!(query.at(query.today, AgendaSpan::Days(0)).is_err());
}
// A block plan applies native metadata filtering, property columns and
// duration-aware ordering to real canonical text while keeping source offsets.
#[test]
fn custom_blocks() {
    let source = "* TODO Small :work:\n:PROPERTIES:\n:EFFORT: 0:30\n:OWNER: Ada\n:END:\n* TODO Large :work:\n:PROPERTIES:\n:EFFORT: 2:00\n:END:\n<2024-03-01 Fri>\n* TODO Home :home:\n";
    let f = Fixture::new();
    let startup = prepare(&f.write("config.toml", "org.time.zone='UTC'"));
    let policy = startup.snapshot.settings().org.clone();
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&f.write("tasks.org", source)).unwrap();
    let selection = SourceSelection {
        paths: vec![],
        buffers: vec![id],
    };
    let sources = SourceScan::new(&buffers, selection, &policy.limits)
        .unwrap()
        .run(&Cancellation::default());
    let view: AgendaView=toml::from_str("key='w'\ntitle='Work'\nblocks=[{kind='todo',filter='work',columns=['ITEM','EFFORT','OWNER'],sort=['effort-down']},{kind='calendar',span='day',filter='work'}]").unwrap();
    let context = CalendarQuery::new(super::habits::clock(&policy.time).unwrap(), &policy).unwrap();
    let result =
        CustomView::observe(&sources, view, context, &policy, &Cancellation::default()).unwrap();
    assert!(result.complete(), "{:?}", result.issues);
    assert_eq!(result.rows.len(), 3);
    assert_eq!(result.rows[0].title.as_ref(), "Large");
    assert_eq!(result.rows[1].fields["OWNER"], "Ada");
    assert_eq!(result.rows[2].fields["DATE"], "2024-03-01");
    assert_eq!(result.rows[0].byte, source.find("* TODO Large").unwrap());
    assert_eq!(sources.documents[0].text().to_string(), source);
}

fn evaluate(
    config: &str,
    source: &str,
    blocks: Vec<rev::config::AgendaBlock>,
) -> rev::org::agenda::QueryResult {
    let f = Fixture::new();
    let startup = prepare(&f.write("config.toml", &format!("org.time.zone='UTC'\n{config}")));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let policy = startup.snapshot.settings().org.clone();
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&f.write("tasks.org", source)).unwrap();
    let sources = SourceScan::new(
        &buffers,
        SourceSelection {
            paths: vec![],
            buffers: vec![id],
        },
        &policy.limits,
    )
    .unwrap()
    .run(&Cancellation::default());
    let context = CalendarQuery::new(super::habits::clock(&policy.time).unwrap(), &policy).unwrap();
    CustomView::observe(
        &sources,
        AgendaView {
            key: "test".into(),
            title: "Test".into(),
            blocks,
        },
        context,
        &policy,
        &Cancellation::default(),
    )
    .unwrap()
}
// Global and block row limits compose without claiming exact-bound results are
// incomplete; 32 blocks share one source table and one global row budget.
#[test]
fn custom_combination_limits() {
    use rev::config::AgendaBlock;
    for (name, blocks, maximum, rows, complete) in [
        ("exact", 2, 4, 4, true),
        ("over", 2, 3, 3, false),
        ("maximum-blocks", 32, 64, 64, true),
    ] {
        let result = evaluate(
            &format!("org.limits.agenda_rows={maximum}"),
            "* TODO A\n* TODO B\n",
            vec![AgendaBlock::default(); blocks],
        );
        assert_eq!(result.rows.len(), rows, "{name}");
        assert_eq!(result.complete(), complete, "{name}: {:?}", result.issues);
        assert_eq!(result.sources.len(), 1, "{name}");
        assert!(result.rows.iter().all(|r| r.source == 0));
    }
    let result = evaluate(
        "",
        "* TODO A\n* TODO B\n",
        vec![AgendaBlock {
            row_limit: 1,
            ..AgendaBlock::default()
        }],
    );
    assert_eq!(result.rows.len(), 1);
    assert!(!result.complete());
}
// Closed clocks use actual clipped instants rather than their printed duration.
// Literal and unselected-child records cannot leak into the preceding task.
#[test]
fn custom_clock_columns() {
    use rev::config::{AgendaBlock, AgendaKind};
    let source = "* TODO Work :work:\n:LOGBOOK:\nCLOCK: [2024-02-29 Thu 23:30]--[2024-03-01 Fri 00:30] => 9:00\nCLOCK: [2024-03-01 Fri 10:00]\n:END:\n#+begin_example\nCLOCK: [2024-03-01 Fri 10:00]--[2024-03-01 Fri 11:00] => 1:00\n#+end_example\n** TODO Child :home:\nCLOCK: [2024-03-01 Fri 10:00]--[2024-03-01 Fri 11:00] => 1:00\n";
    let block = AgendaBlock {
        kind: AgendaKind::Clock,
        query: "TAGS=\":work:\"".into(),
        span: Some(AgendaSpan::Day),
        columns: vec!["CLOCKSUM".into(), "CLOCKOPEN".into()],
        ..AgendaBlock::default()
    };
    let result = evaluate("", source, vec![block]);
    assert!(result.complete(), "{:?}", result.issues);
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].fields["CLOCKSUM"], "30");
    assert_eq!(result.rows[0].fields["CLOCKOPEN"], "1");
}
// Explicit hardlink and symlink block roots retain membership after canonical
// deduplication, including unsaved canonical buffer precedence.
#[test]
fn custom_source_aliases() {
    use rev::config::{AgendaBlock, AgendaKind};
    let f = Fixture::new();
    let path = f.write("tasks.org", "* TODO Saved\n");
    let alias = f.0.join("alias.org");
    std::fs::hard_link(&path, &alias).unwrap();
    let link = f.0.join("link.org");
    std::os::unix::fs::symlink(&path, &link).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    // Native edits, rather than disk replacement, are authoritative for all aliases.
    let buffer = buffers.get_mut(id).unwrap();
    let end = buffer.len_chars();
    let edits = [rev::buffer::edit::Replacement {
        source: buffer.span(end..end).unwrap(),
        text: "* TODO Unsaved\n".into(),
    }];
    let prepared = buffer.prepare(&edits, &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    let policy = rev::config::Org::default();
    let sources = SourceScan::new(
        &buffers,
        SourceSelection {
            paths: vec![path, alias.clone(), link.clone()],
            buffers: vec![],
        },
        &policy.limits,
    )
    .unwrap()
    .run(&Cancellation::default());
    assert_eq!(sources.documents.len(), 1);
    let blocks = [alias.clone(), link]
        .into_iter()
        .map(|path| AgendaBlock {
            kind: AgendaKind::Todo,
            sources: vec![path.to_string_lossy().into_owned()],
            ..AgendaBlock::default()
        })
        .collect();
    let context = CalendarQuery::new(super::habits::clock(&policy.time).unwrap(), &policy).unwrap();
    let result = CustomView::observe(
        &sources,
        AgendaView {
            key: "a".into(),
            title: "Aliases".into(),
            blocks,
        },
        context,
        &policy,
        &Cancellation::default(),
    )
    .unwrap();
    assert!(result.complete(), "{:?}", result.issues);
    assert_eq!(result.rows.len(), 4);
    assert_eq!(result.sources.len(), 1);
    assert_eq!(std::fs::read_to_string(alias).unwrap(), "* TODO Saved\n");
}

// Search, property and column blocks retain the same native query grammar and
// derived-field rules; column text comes from the canonical property resolver.
#[test]
fn custom_query_kinds() {
    use rev::config::{AgendaBlock, AgendaKind as K};
    for (kind, query) in [
        (K::Search, "Needle"),
        (K::Properties, "OWNER=\"Ada\""),
        (K::Columns, "OWNER=\"Ada\""),
    ] {
        let result = evaluate(
            "",
            "* TODO Needle\n:PROPERTIES:\n:OWNER: Ada\n:END:\n* TODO Other\n",
            vec![AgendaBlock {
                kind,
                query: query.into(),
                columns: vec!["OWNER".into()],
                ..AgendaBlock::default()
            }],
        );
        assert!(result.complete(), "{kind:?}: {:?}", result.issues);
        assert_eq!(result.rows.len(), 1, "{kind:?}");
        assert_eq!(result.rows[0].fields["OWNER"], "Ada");
    }
}

// Directory aliases preserve block scope without rereading source bytes; the
// original global 65536 directory-entry budget covers all selected aliases.
#[test]
fn custom_directory_alias() {
    use rev::config::AgendaBlock;
    let f = Fixture::new();
    f.write("folder/task.org", "* TODO One\n");
    let root = f.0.join("folder");
    let alias = f.0.join("linked");
    std::os::unix::fs::symlink(&root, &alias).unwrap();
    let policy = rev::config::Org::default();
    let buffers = Buffers::new().unwrap();
    let sources = SourceScan::new(
        &buffers,
        SourceSelection {
            paths: vec![root.clone(), alias.clone()],
            buffers: vec![],
        },
        &policy.limits,
    )
    .unwrap()
    .run(&Cancellation::default());
    assert_eq!(sources.documents.len(), 1);
    let blocks = [root, alias]
        .into_iter()
        .map(|p| AgendaBlock {
            sources: vec![p.to_string_lossy().into_owned()],
            ..AgendaBlock::default()
        })
        .collect();
    let context = CalendarQuery::new(super::habits::clock(&policy.time).unwrap(), &policy).unwrap();
    let result = CustomView::observe(
        &sources,
        AgendaView {
            key: "d".into(),
            title: "Directories".into(),
            blocks,
        },
        context,
        &policy,
        &Cancellation::default(),
    )
    .unwrap();
    assert!(result.complete(), "{:?}", result.issues);
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.sources.len(), 1);
}
