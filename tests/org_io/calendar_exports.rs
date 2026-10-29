use super::*;
use rev::{
    config::{ExportBackend, Org},
    org::{
        Cancellation,
        export::{Request, Scope, Work},
    },
};
fn render(source: &str) -> Result<String, String> {
    Ok(render_at(source, clock(&Org::default().time).unwrap())?
        .text()
        .into())
}
fn render_at(source: &str, time: ZonedOperation) -> Result<rev::org::export::Rendered, String> {
    let f = Fixture::new();
    let path = f.write("calendar.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let request = Request {
        backend: ExportBackend::Icalendar,
        scope: Scope::Buffer,
        options: None,
        time,
    };
    let snapshot =
        Work::new(&buffers, id, request, &Org::default())?.run(&Cancellation::default())?;
    snapshot.render(&Cancellation::default())
}
fn in_zone(spec: &str) -> ZonedOperation {
    use rev::org::time::{ResolvedTime, Zone};
    let zone = Zone::from_posix(spec).unwrap();
    let unix_seconds = 1709303437;
    ZonedOperation::from_resolved(
        ResolvedTime {
            unix_seconds,
            offset_seconds: zone.offset_at(unix_seconds).unwrap(),
        },
        zone,
    )
    .unwrap()
}
// Native event/task mapping uses checked dates, exclusive all-day ends and
// one captured UTC operation. No source-looking line becomes a calendar field.
#[test]
fn calendar_native_document() {
    let source = "#+TITLE: Planning\n#+OPTIONS: todo:nil tags:nil\n* TODO Write\nSCHEDULED: <2024-03-04 Mon 09:00> DEADLINE: <2024-03-04 Mon 17:00>\n:PROPERTIES:\n:ID: write-task\n:LOCATION: Desk, room; north\\side\n:END:\nText\n* Meeting\n:PROPERTIES:\n:ID: meeting\n:END:\n<2024-03-05 Tue 10:00-11:00>\n* Vacation\n<2024-03-06 Wed>--<2024-03-08 Fri>\n#+begin_src sh\ntouch marker\n<2024-01-01 Mon>\n#+end_src\n";
    let output = render(source).unwrap().replace("\r\n ", "");
    assert!(output.starts_with("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n"));
    assert!(output.ends_with("END:VCALENDAR\r\n"));
    assert_eq!(output.matches("BEGIN:VEVENT\r\n").count(), 2);
    assert_eq!(output.matches("BEGIN:VTODO\r\n").count(), 1);
    for value in [
        "SUMMARY:Write",
        "DTSTART:20240304T090000Z",
        "DUE:20240304T170000Z",
        "DTEND:20240305T110000Z",
        "DTEND;VALUE=DATE:20240309",
        "LOCATION:Desk\\, room\\; north\\\\side",
        "touch marker\\n<2024-01-01 Mon>",
    ] {
        assert!(output.contains(value), "{value}: {output}");
    }
    assert!(!output.contains("DTSTART;VALUE=DATE:20240101"));
}

// Only interval rules with the same native sequence become RRULE. Unsupported
// completion/habit modes and month normalization retain their complete spelling.
#[test]
fn calendar_recurrence_profile() {
    for (source, rule) in [
        ("<2024-03-04 Mon +2w>", "FREQ=WEEKLY;INTERVAL=2"),
        ("<2024-03-04 Mon 09:00 +3h>", "FREQ=HOURLY;INTERVAL=3"),
        ("<2024-03-04 Mon +1m>", "FREQ=MONTHLY;INTERVAL=1"),
        ("<2024-03-04 Mon +1y>", "FREQ=YEARLY;INTERVAL=1"),
    ] {
        let output = render(&format!("* Entry\n{source}\n")).unwrap();
        assert!(output.contains(&format!("RRULE:{rule}\r\n")), "{output}");
    }
    for source in [
        "<2024-03-04 Mon ++1w>",
        "<2024-03-04 Mon .+1d>",
        "<2024-03-04 Mon +1d/2d>",
        "<2024-01-31 Wed +1m>",
        "<2024-02-29 Thu +1y>",
    ] {
        let output = render(&format!("* Entry\n{source}\n"))
            .unwrap()
            .replace("\r\n ", "");
        assert!(output.contains(&format!("X-ORG-TIMESTAMP:{source}")));
        assert!(output.contains("X-ORG-RECURRENCE-LIMIT:"));
        assert!(!output.contains("\r\nRRULE:"));
    }
}

// Owned POSIX observations prove date-specific UTC conversion without host TZ
// dependence. Source-selected UTC overrides the retained default operation zone.
#[test]
fn calendar_zone_conversion() {
    let source = "* Before\n<2024-03-09 Sat 09:00-10:00>\n* After\n<2024-03-11 Mon 09:00-10:00>\n* UTC\n:PROPERTIES:\n:TIMEZONE: UTC\n:END:\n<2024-03-11 Mon 09:00-10:00>\n";
    let output = render_at(source, in_zone("EST5EDT,M3.2.0/2,M11.1.0/2")).unwrap();
    for expected in [
        "DTSTART:20240309T140000Z",
        "DTEND:20240309T150000Z",
        "DTSTART:20240311T130000Z",
        "DTEND:20240311T140000Z",
        "DTSTART:20240311T090000Z",
    ] {
        assert!(output.text().contains(expected), "{}", output.text());
    }
    let range = render("* Range\n<2024-03-04 Mon 09:00>--<2024-03-05 Tue 10:00-11:00>\n").unwrap();
    assert!(range.contains("DTEND:20240305T110000Z"));
    for (name, source, expected) in [
        ("gap", "* Gap\n<2024-03-10 Sun 02:30>\n", "Nonexistent"),
        ("fold", "* Fold\n<2024-11-03 Sun 01:30>\n", "Ambiguous"),
    ] {
        let error = render_at(source, in_zone("EST5EDT,M3.2.0/2,M11.1.0/2"))
            .err()
            .unwrap();
        assert!(error.contains(expected), "{name}: {error}");
    }
}
// A constant offset is proved from retained rules, not sampled clock readings.
// Variable-zone recurrence and UTC calendar-field changes never invent a series.
#[test]
fn calendar_zone_recurrence() {
    for (name, source, zone, supported) in [
        ("fixed weekly", "<2024-03-09 Sat 09:00 +1w>", "EST5", true),
        (
            "variable weekly",
            "<2024-03-09 Sat 09:00 +1w>",
            "EST5EDT,M3.2.0/2,M11.1.0/2",
            false,
        ),
        (
            "date only",
            "<2024-03-09 Sat +1w>",
            "EST5EDT,M3.2.0/2,M11.1.0/2",
            true,
        ),
        (
            "calendar day preserved",
            "<2024-03-09 Sat 09:00 +1m>",
            "EST5",
            true,
        ),
        (
            "calendar day changes",
            "<2024-03-09 Sat 23:00 +1m>",
            "EST5",
            false,
        ),
    ] {
        let result = render_at(&format!("* Entry\n{source}\n"), in_zone(zone)).unwrap();
        let text = result.text().replace("\r\n ", "");
        assert_eq!(text.contains("\r\nRRULE:"), supported, "{name}");
        assert_eq!(
            result.diagnostics().is_empty(),
            supported,
            "{name}: {:?}",
            result.diagnostics()
        );
        assert!(text.contains(source), "{name}");
    }
}
// Task recurrence needs one DTSTART anchor; independent due/range/completion
// semantics stay explicit source and diagnosed limits instead of new dates.
#[test]
fn calendar_task_limits() {
    for (name, planning, supported) in [
        ("scheduled", "SCHEDULED: <2024-03-04 Mon +1w>", true),
        ("due only", "DEADLINE: <2024-03-04 Mon +1w>", false),
        (
            "two anchors",
            "SCHEDULED: <2024-03-04 Mon +1w> DEADLINE: <2024-03-05 Tue +1w>",
            false,
        ),
        ("restart", "SCHEDULED: <2024-03-04 Mon .+1w>", false),
    ] {
        let result = render_at(
            &format!("* TODO Entry\n{planning}\n"),
            clock(&Org::default().time).unwrap(),
        )
        .unwrap();
        assert_eq!(result.text().contains("\r\nRRULE:"), supported, "{name}");
        assert_eq!(result.diagnostics().is_empty(), supported, "{name}");
        assert!(
            result.text().replace("\r\n ", "").contains(planning),
            "{name}"
        );
    }
    let result = render_at(
        "* TODO Entry\nSCHEDULED: <2024-03-04 Mon 09:00-10:00 --2d>\n",
        clock(&Org::default().time).unwrap(),
    )
    .unwrap();
    assert!(result.diagnostics().iter().any(|d| d.contains("extent")));
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|d| d.contains("warning delay"))
    );
    assert!(!result.text().contains("VALARM"));
}
// Invalid native syntax, duplicate identities and incompatible date/time bounds
// reject the candidate before any destination effect or source normalization.
#[test]
fn calendar_rejected_source() {
    for (name, source, expected) in [
        (
            "duplicate ID",
            "* A\n:PROPERTIES:\n:ID: same\n:END:\n<2024-03-01 Fri>\n* B\n:PROPERTIES:\n:ID: same\n:END:\n<2024-03-02 Sat>\n",
            "Duplicate calendar entry identity",
        ),
        (
            "ambiguous fallback",
            "* TODO Same\n* DONE Same\n",
            "Duplicate calendar entry identity",
        ),
        (
            "mixed bounds",
            "* Entry\n<2024-03-01 Fri 09:00>--<2024-03-02 Sat>\n",
            "matching value types",
        ),
        (
            "reversed task",
            "* TODO Entry\nSCHEDULED: <2024-03-02 Sat> DEADLINE: <2024-03-01 Fri>\n",
            "end after the start",
        ),
        (
            "zero event",
            "* Entry\n<2024-03-01 Fri 09:00-09:00>\n",
            "end after the start",
        ),
        (
            "bad unit",
            "* Entry\n<2024-03-01 Fri +1q>\n",
            "interval requires",
        ),
        ("control", "* TODO Invalid\u{1} scalar\n", "control scalar"),
        (
            "bad zone",
            "* TODO Entry\n:PROPERTIES:\n:TIMEZONE: ../escape\n:END:\n",
            "cannot contain",
        ),
        (
            "no data",
            "* Entry\n[2024-03-01 Fri]\n~<2024-03-02 Sat>~\n",
            "No native Org calendar entries",
        ),
    ] {
        assert!(
            render(source).unwrap_err().contains(expected),
            "{name}: {:?}",
            render(source)
        );
    }
}
