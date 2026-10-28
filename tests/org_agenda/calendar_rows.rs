use super::*;
use rev::{
    config::AgendaSpan,
    org::{
        Cancellation,
        agenda::{
            Calendar, CalendarKind as K, CalendarLine, CalendarQuery, SourceScan, SourceSelection,
        },
        time::Date,
    },
};
fn observe(config: &str, source: &str) -> (Fixture, Calendar) {
    let f = Fixture::new();
    let config = prepare(&f.write("config.toml", config));
    assert!(config.diagnostic.is_none(), "{:?}", config.diagnostic);
    let policy = config.snapshot.settings().org.clone();
    let mut buffers = Buffers::with_settings(config.snapshot).unwrap();
    let path = f.write("calendar.org", source);
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
    let query = CalendarQuery::new(super::habits::clock(&policy.time).unwrap(), &policy).unwrap();
    (
        f,
        Calendar::observe(&sources, query, &policy, &Cancellation::default()).unwrap(),
    )
}
// Day/week projection uses the captured effective date, planning semantics and
// real source identities. Querying never changes the original source bytes.
#[test]
fn calendar_planning() {
    let source = "* TODO Late schedule\nSCHEDULED: <2024-02-27 Tue>\n* TODO Late deadline\nDEADLINE: <2024-02-28 Wed>\n* TODO Soon\nDEADLINE: <2024-03-04 Mon>\n* Meet 界\n<2024-03-01 Fri 09:00-10:00>\n* DONE Finished\nSCHEDULED: <2024-02-26 Mon>\n";
    let (f, view) = observe("org.time.zone='UTC'", source);
    assert!(view.complete(), "{:?}", view.issues);
    assert_eq!(view.query.start, Date::new(2024, 2, 26).unwrap());
    for kind in [
        K::OverdueScheduled,
        K::OverdueDeadline,
        K::Warning,
        K::Scheduled,
        K::Deadline,
        K::Appointment,
    ] {
        assert!(view.rows.iter().any(|r| r.kind == kind), "{kind:?}");
    }
    assert_eq!(
        view.rows
            .iter()
            .filter(|r| r.title.contains("Finished"))
            .count(),
        1
    );
    assert!(
        view.lines
            .iter()
            .any(|l| matches!(l,CalendarLine::Grid(t) if t.hour()==10))
    );
    let day = view.project(
        view.query.at(view.query.today, AgendaSpan::Day).unwrap(),
        &Cancellation::default(),
    );
    assert!(day.rows.iter().all(|r| r.date == day.query.today));
    assert_eq!(
        day.query.operation.instant(),
        view.query.operation.instant()
    );
    let next = day.project(day.query.shifted(1).unwrap(), &Cancellation::default());
    assert_eq!(next.query.start, Date::new(2024, 3, 2).unwrap());
    assert_eq!(
        std::fs::read_to_string(f.0.join("calendar.org")).unwrap(),
        source
    );
}
// Existing structural recognition excludes literal blocks, metadata drawers,
// COMMENT/archive subtrees and inactive dates; local workflows control reminders.
#[test]
fn calendar_source_contexts() {
    let source = "#+TODO: WAIT | CLOSED\n* CLOSED Old\nSCHEDULED: <2024-02-01 Thu>\n* COMMENT Hidden\n<2024-03-01 Fri>\n** Hidden child\n<2024-03-01 Fri>\n* Archived :ARCHIVE:\n<2024-03-01 Fri>\n* Visible\n:PROPERTIES:\n:WHEN: <2024-03-01 Fri>\n:END:\n#+BEGIN_SRC text\n,* Fake\n<2024-03-01 Fri>\n#+END_SRC\n[2024-03-01 Fri]\n<2024-02-29 Thu>--<2024-03-02 Sat>\n* Weekly\n<2024-02-23 Fri 11:00 +1w>\n";
    let (_f, view) = observe("org.time.zone='UTC'", source);
    assert!(view.complete(), "{:?}", view.issues);
    assert_eq!(view.rows.len(), 4, "{:?}", view.rows);
    assert_eq!(
        view.rows
            .iter()
            .filter(|r| r.title.as_ref() == "Visible")
            .count(),
        3
    );
    assert_eq!(
        view.rows
            .iter()
            .filter(|r| r.title.as_ref() == "Weekly")
            .count(),
        1
    );
    assert!(view.rows.iter().all(|r| r.byte > 0));
}
// Exact row/fact admission stays complete. Excess, malformed dates and partial
// sources remain visible rather than becoming a complete empty calendar.
#[test]
fn calendar_limits() {
    for (name, config, source, count, complete) in [
        (
            "exact",
            "org.limits.agenda_rows=1",
            "* One\n<2024-03-01 Fri>",
            1,
            true,
        ),
        (
            "rows",
            "org.limits.agenda_rows=1",
            "* Range\n<2024-03-01 Fri>--<2024-03-02 Sat>",
            1,
            false,
        ),
        (
            "facts",
            "org.limits.agenda_rows=1",
            "* One\n<2024-03-01 Fri>\n* Two\n<2024-03-02 Sat>",
            1,
            false,
        ),
        ("malformed", "", "* One\n<2024-02-30 Fri>", 0, false),
        ("inactive", "", "* One\n[2024-03-01 Fri]", 0, true),
        ("angle", "", "* One\n<https://example.invalid", 0, true),
    ] {
        let (_f, view) = observe(config, source);
        assert_eq!(view.rows.len(), count, "{name}");
        assert_eq!(view.complete(), complete, "{name}: {:?}", view.issues);
    }
    let (_f, view) = observe("org.agenda.week_start=7", "* One\n<2024-03-01 Fri>");
    assert_eq!(view.query.start, Date::new(2024, 2, 25).unwrap());
    let cancel = Cancellation::default();
    cancel.cancel();
    let cancelled = view.project(view.query.clone(), &cancel);
    assert!(cancelled.cancelled);
    assert!(!cancelled.complete());
}
