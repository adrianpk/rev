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

fn verify(path: &Path, fixture: bool) {
    let mut command = std::process::Command::new("python3");
    command.args([
        "scripts/configuration_ui/calendar_document.py",
        path.to_str().unwrap(),
    ]);
    if fixture {
        command.arg("--fixture");
    }
    let check = command.output().unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}
// Python's independent date/wire reader checks actual emitted bytes against
// hand-authored semantic fixtures, including leap days and inert protocol text.
#[test]
fn calendar_independent_fixture() {
    let f = Fixture::new();
    let output = render(include_str!("../fixtures/org-calendar/document.org")).unwrap();
    let path = f.write("document.ics", &output);
    verify(&path, true);
    assert!(!f.0.join("marker").exists());
}
// Folding counts encoded octets and never tears UTF-8 scalars. Escape spelling
// is independently unfolded/decoded and cannot inject calendar components.
#[test]
fn calendar_unicode_folding() {
    let f = Fixture::new();
    let title = format!("Text {} ,;\\ tail", "界".repeat(40));
    let output = render(&format!(
        "* TODO {title}\n:PROPERTIES:\n:ID: unicode\n:END:\nEND:VCALENDAR\nBEGIN:VEVENT\n"
    ))
    .unwrap();
    let path = f.write("document.ics", &output);
    verify(&path, false);
    let check = std::process::Command::new("python3")
        .args([
            "-c",
            r#"import sys
from pathlib import Path
from scripts.configuration_ui.calendar_document import Document, one
d = Document(Path(sys.argv[1]).read_bytes())
assert len(d.components) == 1
assert one(d.components[0], 'SUMMARY') == 'TODO Text ' + '界' * 40 + ' ,;\\ tail'
assert one(d.components[0], 'DESCRIPTION') == 'END:VCALENDAR\nBEGIN:VEVENT'
"#,
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}
fn begin(session: &mut Session, destination: &str) {
    action(session, C::ExportIcalendar);
    for value in ["buffer", "todo:nil tags:nil", destination] {
        argument(session, value);
    }
}
// Dirty BOM/CRLF source and ordinary undo retain their revision, history, view
// and disk ownership while the explicit command publishes exact CRLF output.
#[test]
fn calendar_file_publication() {
    let f = Fixture::new();
    let source = "\u{feff}* TODO Heading\r\nBody";
    let path = f.write("source.org", source);
    let mut s = host(&f, Some(&path), false);
    s.handle(Event::Paste("#+TITLE: Unsaved\r\n".into()));
    settle(&mut s);
    let before = text(&s);
    let view = s.view().clone();
    let stamp = s.buffer().unwrap().stamp();
    let history = s.buffer().unwrap().history().len();
    begin(&mut s, "document.ics");
    assert!(s.echo().contains("Exported Org document"), "{}", s.echo());
    verify(&f.0.join("document.ics"), false);
    assert_eq!(text(&s), before);
    assert_eq!(s.view(), &view);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    assert!(s.buffer().unwrap().is_modified());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    s.execute(Command::History(rev::command::History::Undo));
    assert_eq!(text(&s), "* TODO Heading\r\nBody");
}
// Prompt cancellation, declined replacement, stale source/include/destination
// observations and aliases retain the existing file and never save the source.
#[test]
fn calendar_publication_guards() {
    let (f, mut s) = super::links::editor(
        "org.tools.pdf=['./absent','{input}']",
        "#+INCLUDE: \"child.org\"\n* TODO Source\n",
        "",
    );
    f.write("child.org", "* TODO Child\n");
    let output = f.write("document.ics", "keep");
    for count in 0..4 {
        action(&mut s, C::ExportIcalendar);
        for value in ["buffer", "", "document.ics"].into_iter().take(count) {
            argument(&mut s, value);
        }
        cancel(&mut s);
        settle(&mut s);
        assert_eq!(std::fs::read(&output).unwrap(), b"keep");
    }
    begin(&mut s, "document.ics");
    argument(&mut s, "no");
    assert_eq!(std::fs::read(&output).unwrap(), b"keep");
    begin(&mut s, "document.ics");
    f.write("document.ics", "changed");
    argument(&mut s, "yes");
    assert!(s.echo().contains("not published"), "{}", s.echo());
    assert_eq!(std::fs::read(&output).unwrap(), b"changed");
    begin(&mut s, "document.ics");
    f.write("child.org", "* TODO Changed\n");
    argument(&mut s, "yes");
    assert!(s.echo().contains("not published"), "{}", s.echo());
    assert_eq!(std::fs::read(&output).unwrap(), b"changed");
    begin(&mut s, "document.ics");
    s.insert('X');
    argument(&mut s, "yes");
    assert!(s.echo().contains("source or view changed"), "{}", s.echo());
    assert_eq!(std::fs::read(&output).unwrap(), b"changed");
    std::fs::hard_link(f.0.join("source.org"), f.0.join("alias")).unwrap();
    for name in ["source.org", "child.org", "alias"] {
        let before = std::fs::read(f.0.join(name)).unwrap();
        begin(&mut s, name);
        assert!(!s.echo().contains("Exported Org document"));
        assert_eq!(std::fs::read(f.0.join(name)).unwrap(), before);
    }
}
// Selected read-only heading scopes retain explicit identity and suppress file
// metadata in regions. Source range coordinates still belong to the editor.
#[test]
fn calendar_selected_scope() {
    let f = Fixture::new();
    let source = "#+TITLE: Hidden\n* TODO Parent\n** TODO Child :work:\n:PROPERTIES:\n:ID: child\n:END:\nbody\n** TODO Sibling\n";
    let path = f.write("source.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
    .with_org_clock(clock);
    s.execute(Command::GotoLine);
    argument(&mut s, "3");
    let view = s.view().clone();
    action(&mut s, C::ExportIcalendar);
    for value in ["subtree", "title:nil todo:nil tags:nil", "child.ics"] {
        argument(&mut s, value);
    }
    assert!(s.echo().contains("Exported Org document"), "{}", s.echo());
    let output = std::fs::read_to_string(f.0.join("child.ics")).unwrap();
    for hidden in ["Hidden", "Parent", "Sibling", ":work:"] {
        assert!(!output.contains(hidden), "{hidden}");
    }
    assert!(output.contains("SUMMARY:Child\r\n"));
    assert!(!output.contains("SUMMARY:TODO"));
    assert_eq!(s.view(), &view);
    s.execute(Command::GotoLine);
    argument(&mut s, "3");
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char(' '),
        KeyModifiers::CONTROL,
    )));
    s.execute(Command::GotoLine);
    argument(&mut s, "8");
    let view = s.view().clone();
    action(&mut s, C::ExportIcalendar);
    for value in ["region", "todo:nil tags:nil", "region.ics"] {
        argument(&mut s, value);
    }
    assert!(s.echo().contains("Exported Org document"), "{}", s.echo());
    let region = std::fs::read_to_string(f.0.join("region.ics")).unwrap();
    assert_eq!(output, region);
    assert_eq!(s.view(), &view);
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
}
// Fresh preparation is stable for explicit and fallback identities despite
// unrelated prefix edits and task-state changes. Rendering itself is repeatable.
#[test]
fn calendar_stable_refresh() {
    let f = Fixture::new();
    let source = "* TODO Fallback\n* TODO Explicit\n:PROPERTIES:\n:ID: stable-id\n:END:\n";
    let path = f.write("source.org", source);
    let mut identities = None;
    for value in [
        source.to_string(),
        source.to_string(),
        format!("#+TITLE: Changed\n{}", source.replace("TODO", "DONE")),
    ] {
        std::fs::write(&path, value).unwrap();
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&path).unwrap();
        let request = Request {
            backend: ExportBackend::Icalendar,
            scope: Scope::Buffer,
            options: None,
            time: clock(&Org::default().time).unwrap(),
        };
        let snapshot = Work::new(&buffers, id, request, &Org::default())
            .unwrap()
            .run(&Cancellation::default())
            .unwrap();
        let first = snapshot.render(&Cancellation::default()).unwrap();
        assert_eq!(
            first.text(),
            snapshot.render(&Cancellation::default()).unwrap().text()
        );
        let ids: Vec<_> = first
            .text()
            .lines()
            .filter(|line| line.starts_with("UID:"))
            .map(str::to_owned)
            .collect();
        if let Some(previous) = &identities {
            assert_eq!(&ids, previous);
        } else {
            identities = Some(ids);
        }
    }
}
// Exact wire byte budgets include folds and CRLF; cancelled or excessive work
// never returns a partial calendar. Component count remains a separate bound.
#[test]
fn calendar_output_bounds() {
    let f = Fixture::new();
    let source = "* TODO Bound\n:PROPERTIES:\n:ID: bound\n:END:\nbody\n";
    let path = f.write("source.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let expected = render(source).unwrap();
    for limit in [expected.len(), expected.len() - 1] {
        let mut policy = Org::default();
        policy.limits.export_bytes = limit;
        let request = Request {
            backend: ExportBackend::Icalendar,
            scope: Scope::Buffer,
            options: None,
            time: clock(&policy.time).unwrap(),
        };
        let snapshot = Work::new(&buffers, id, request, &policy)
            .unwrap()
            .run(&Cancellation::default())
            .unwrap();
        let result = snapshot.render(&Cancellation::default());
        if limit == expected.len() {
            assert_eq!(result.unwrap().text(), expected);
        } else {
            assert!(result.err().unwrap().contains("byte limit"));
        }
        let cancel = Cancellation::default();
        cancel.cancel();
        assert!(snapshot.render(&cancel).is_err());
    }
    let source = format!(
        "* TODO Many\n:PROPERTIES:\n:ID: many\n:DESCRIPTION: bounded\n:END:\n{}",
        "<2024-03-04 Mon> ".repeat(4096)
    );
    assert!(render(&source).unwrap_err().contains("4096 components"));
}
