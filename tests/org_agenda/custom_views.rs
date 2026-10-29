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
