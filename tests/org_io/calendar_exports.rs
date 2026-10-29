use super::*;
use rev::{
    config::{ExportBackend, Org},
    org::{
        Cancellation,
        export::{Request, Scope, Work},
    },
};
fn render(source: &str) -> Result<String, String> {
    let f = Fixture::new();
    let path = f.write("calendar.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let request = Request {
        backend: ExportBackend::Icalendar,
        scope: Scope::Buffer,
        options: None,
        time: clock(&Org::default().time).unwrap(),
    };
    let snapshot =
        Work::new(&buffers, id, request, &Org::default())?.run(&Cancellation::default())?;
    Ok(snapshot.render(&Cancellation::default())?.text().into())
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
