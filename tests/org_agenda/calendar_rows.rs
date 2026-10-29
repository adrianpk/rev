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
    observe_with(config, source, &Cancellation::default())
}
fn observe_with(config: &str, source: &str, cancel: &Cancellation) -> (Fixture, Calendar) {
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
        Calendar::observe(&sources, query, &policy, cancel).unwrap(),
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
    let (_f, partial) = observe_with("", "* One\n<2024-03-01 Fri>", &cancel);
    let retained = partial.project(partial.query.clone(), &Cancellation::default());
    assert!(
        !retained.complete(),
        "cancelled facts cannot become complete on date movement"
    );
}
// A range retains its endpoint times on their actual dates. Continuation days
// cannot invent daily starts, and same-day paired timestamps retain their end.
#[test]
fn calendar_range_endpoints() {
    for (source, expected) in [
        (
            "* Range\n<2024-02-29 Thu 09:00>--<2024-03-02 Sat 10:00>",
            vec![(29, Some(9), None), (1, None, None), (2, None, Some(10))],
        ),
        (
            "* Overnight\n<2024-02-29 Thu 23:00-01:00>",
            vec![(29, Some(23), None), (1, None, Some(1))],
        ),
        (
            "* Paired\n<2024-03-01 Fri 09:00>--<2024-03-01 Fri 11:00>",
            vec![(1, Some(9), Some(11))],
        ),
    ] {
        let (_f, view) = observe("org.time.zone='UTC'", source);
        assert!(view.complete(), "{:?}", view.issues);
        let actual: Vec<_> = view
            .rows
            .iter()
            .map(|r| {
                (
                    r.date.day(),
                    r.time.map(|t| t.hour()),
                    r.end_time.map(|t| t.hour()),
                )
            })
            .collect();
        assert_eq!(actual, expected, "{source}");
    }
}
// Captured zone rules diagnose both endpoint gaps/folds without selecting an
// offset. Effective-today extension changes the query day, never source dates.
#[test]
fn calendar_zone_context() {
    use rev::org::time::{ResolvedTime, Zone, ZonedOperation};
    let (_f, base) = observe(
        "org.time.zone='UTC'",
        "* Gap\n<2024-03-10 Sun 02:30>\n* End gap\n<2024-03-10 Sun 01:30-02:30>\n* Unique\n<2024-03-10 Sun 03:30>\n* Fold\n<2024-11-03 Sun 01:30>",
    );
    let operation = ZonedOperation::from_resolved(
        ResolvedTime {
            unix_seconds: 1709303400,
            offset_seconds: -18000,
        },
        Zone::from_posix("EST5EDT,M3.2.0,M11.1.0").unwrap(),
    )
    .unwrap();
    for (date, expected) in [
        (
            Date::new(2024, 3, 10).unwrap(),
            vec![("End gap", true), ("Gap", true), ("Unique", false)],
        ),
        (Date::new(2024, 11, 3).unwrap(), vec![("Fold", true)]),
    ] {
        let mut query = base.query.at(date, AgendaSpan::Day).unwrap();
        query.operation = std::sync::Arc::new(operation.clone());
        let view = base.project(query, &Cancellation::default());
        assert!(view.complete(), "{:?}", view.issues);
        let mut actual: Vec<_> = view
            .rows
            .iter()
            .map(|r| (r.title.as_ref(), r.unresolved))
            .collect();
        actual.sort();
        assert_eq!(actual, expected);
        assert_eq!(view.query.operation.instant(), operation.instant());
    }
    let (_f, view) = observe(
        "org.time.zone='UTC'\norg.time.extend_today_until=23",
        "* Next day\n<2024-03-01 Fri>",
    );
    assert_eq!(view.query.today, Date::new(2024, 2, 29).unwrap());
    assert_eq!(view.rows[0].date, Date::new(2024, 3, 1).unwrap());
}
// Hourly recurrence preserves the original wall-clock range duration even
// when finding the first visible occurrence skips into the next day.
#[test]
fn calendar_repeating_times() {
    let (_f, view) = observe(
        "org.time.zone='UTC'\norg.agenda.span='day'",
        "* Every six hours\n<2024-02-29 Thu 09:00-10:00 +6h>",
    );
    assert!(view.complete(), "{:?}", view.issues);
    let actual: Vec<_> = view
        .rows
        .iter()
        .map(|r| (r.time.map(|t| t.hour()), r.end_time.map(|t| t.hour())))
        .collect();
    assert_eq!(
        actual,
        vec![
            (Some(3), Some(4)),
            (Some(9), Some(10)),
            (Some(15), Some(16)),
            (Some(21), Some(22))
        ]
    );
}

// Explicit archive inclusion restores tagged parents and descendants while
// COMMENT exclusion remains authoritative. Calendar rows retain source offsets.
#[test]
fn archive_calendar() {
    let source = "* Archived :ARCHIVE:\n<2024-03-01 Fri>\n** Child\n<2024-03-01 Fri>\n* COMMENT Hidden :ARCHIVE:\n<2024-03-01 Fri>\n* Visible\n<2024-03-01 Fri>\n";
    for (name, include, count) in [("default", false, 1), ("included", true, 3)] {
        let (f, view) = observe(
            &format!("org.time.zone='UTC'\norg.agenda.include_archived={include}"),
            source,
        );
        assert!(view.complete(), "{name}: {:?}", view.issues);
        assert_eq!(view.rows.len(), count, "{name}");
        assert!(
            view.rows
                .iter()
                .all(|r| !r.title.contains("Hidden") && source[r.byte..].starts_with('*'))
        );
        assert_eq!(
            std::fs::read_to_string(f.0.join("calendar.org")).unwrap(),
            source
        );
    }
}
