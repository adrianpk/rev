use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{History, Motion, OrgCommand as C};
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
use std::time::{Duration, UNIX_EPOCH};

pub(super) fn clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    ZonedOperation::capture(&ZoneSelection::Utc, &environment("UTC0"), || {
        Ok(UNIX_EPOCH + Duration::from_secs(1793183400))
    })
}
fn environment(tz: &str) -> ZoneEnvironment {
    ZoneEnvironment {
        localtime: "/unused-localtime".into(),
        zoneinfo: "/unused-zoneinfo".into(),
        tz: Some(tz.into()),
    }
}
pub(super) fn dst_clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    ZonedOperation::capture(
        &ZoneSelection::Local,
        &environment("CET-1CEST,M3.5.0,M10.5.0/3"),
        || Ok(UNIX_EPOCH + Duration::from_secs(1729989000)),
    )
}
fn point(session: &mut Session, row: usize, count: usize) {
    goto(session, row);
    for _ in 0..count {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
}
fn erase(session: &mut Session, count: usize) {
    session.handle(key(KeyCode::Home));
    for _ in 0..count {
        session.handle(key(KeyCode::Delete));
    }
}
fn undo(session: &mut Session, source: &str, expected: &str) {
    assert_eq!(text(session), expected, "{}", session.echo());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(session), source);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(session), expected);
}

// The default chord, named command and admitted alias reach one real calendar
// prompt and one canonical transaction; Unicode, CRLF and BOM survive saving.
#[test]
fn timestamp_entry_paths() {
    for path in ["key", "name", "alias"] {
        let source = "* 界\r\ntext";
        let config = "[keys.org]\n'M-j'='org-timestamp'";
        let (fixture, session) = editor(config, "notes.org", &format!("\u{feff}{source}"));
        let mut session = session.with_org_clock(clock);
        point(&mut session, 2, 4);
        let origin = session.view().clone();
        match path {
            "key" => {
                session.handle(control('c'));
                session.handle(key(KeyCode::Char('.')));
            }
            "name" => named(&mut session, "org-timestamp"),
            _ => {
                session.handle(meta('j'));
            }
        }
        settle(&mut session);
        assert_eq!(text(&session), source, "{path}: {}", session.echo());
        assert_eq!(session.view(), &origin);
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|r| r.text().contains("October 2026"))
        );
        argument(&mut session, "2026-10-29 11am-1:15pm");
        let expected = format!("{source}<2026-10-29 Thu 11:00-13:15>");
        undo(&mut session, source, &expected);
        session.execute(Command::SaveBuffer);
        assert_eq!(
            std::fs::read(fixture.0.join("notes.org")).unwrap(),
            format!("\u{feff}{expected}").as_bytes()
        );
    }
}

// Planning edits own only the selected heading's fact. Other planning facts,
// property drawers, unknown text and the document's line ending remain exact.
#[test]
fn planning_timestamps() {
    for (name, source, command, expected) in [
        (
            "new",
            "* H\nbody",
            C::Schedule,
            "* H\nSCHEDULED: <2024-02-29 Thu>\nbody",
        ),
        ("EOF", "* H", C::Deadline, "* H\nDEADLINE: <2024-02-29 Thu>"),
        (
            "neighbors",
            "* H\r\nSCHEDULED: <2024-02-28 Wed> CLOSED: [2024-02-27 Tue]\r\n:PROPERTIES:\r\n:X: keep\r\n:END:\r\n",
            C::Deadline,
            "* H\r\nSCHEDULED: <2024-02-28 Wed> CLOSED: [2024-02-27 Tue] DEADLINE: <2024-02-29 Thu>\r\n:PROPERTIES:\r\n:X: keep\r\n:END:\r\n",
        ),
        (
            "existing",
            "* H\nDEADLINE: <2024-02-28  Mer. 9:05 -2d>\nbody",
            C::Deadline,
            "* H\nDEADLINE: <2024-02-29 Thu 9:05 -2d>\nbody",
        ),
    ] {
        let (_fixture, session) = editor("", "notes.org", source);
        let mut session = session.with_org_clock(clock);
        action(&mut session, command);
        argument(&mut session, "2024-02-29");
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
        undo(&mut session, source, expected);
    }
}

// Calendar navigation borrows presentation rows without moving source anchors.
// C-g discards the entire draft; the saved calendar date can be inserted later.
#[test]
fn calendar_navigation() {
    let source = "* H\nbody\n";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(clock);
    let origin = session.view().clone();
    action(&mut session, C::TimestampInactive);
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Right,
        KeyModifiers::SHIFT,
    )));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Down,
        KeyModifiers::SHIFT,
    )));
    session.handle(Event::Resize(30, 8));
    assert_eq!(session.view(), &origin);
    let frame = session.frame().unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .any(|r| r.text().contains("November 2026"))
    );
    session.handle(control('g'));
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(session.view(), &origin);
    action(&mut session, C::GotoCalendar);
    session.execute(Command::Org(C::CalendarForwardDay));
    session.handle(key(KeyCode::Enter));
    assert_eq!(text(&session), source);
    action(&mut session, C::DateFromCalendar);
    assert_eq!(text(&session), format!("<2026-10-29 Thu>{source}"));
}

// A malformed or Unicode draft stays in the transient owner on rejection and
// never becomes document text. Correcting it commits once; cancellation commits none.
#[test]
fn timestamp_invalid_draft() {
    let source = "* H\n";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(clock);
    action(&mut session, C::TimestampInactive);
    argument(&mut session, "来週");
    assert_eq!(text(&session), source);
    assert!(session.echo().contains("Unknown Org month"));
    erase(&mut session, 2);
    argument(&mut session, "2023-02-29");
    assert_eq!(text(&session), source);
    assert!(session.echo().contains("Gregorian"));
    erase(&mut session, 10);
    argument(&mut session, "2024-02-29");
    undo(&mut session, source, "[2024-02-29 Thu]* H\n");
}

// Editing the selected clock keeps localized weekday/suffix spelling. Editing
// a range's end validates the complete range and retains the first timestamp.
#[test]
fn timestamp_source_spelling() {
    for (name, source, column, draft, expected) in [
        (
            "clock",
            "x <2024-02-29  木  9:05 +1w> y",
            15,
            "10:15",
            "x <2024-02-29  木  10:15 +1w> y",
        ),
        (
            "range end",
            "<2024-02-29 Thu>--<2024-03-01 Fri>",
            20,
            "2024-03-02",
            "<2024-02-29 Thu>--<2024-03-02 Sat>",
        ),
    ] {
        let (_fixture, session) = editor("", "notes.org", source);
        let mut session = session.with_org_clock(clock);
        point(&mut session, 1, column);
        action(&mut session, C::Timestamp);
        argument(&mut session, draft);
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
        undo(&mut session, source, expected);
    }
}

// DST folds require an observed offset choice; gaps return to date entry. Every
// stage remains cancellable, and no canonical edit precedes a complete selection.
#[test]
fn local_time_choices() {
    let source = "* H\n";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(dst_clock);
    action(&mut session, C::Timestamp);
    argument(&mut session, "2024-10-27 02:30");
    assert!(session.echo().contains("Ambiguous"), "{}", session.echo());
    assert!(session.echo().contains("UTC+02:00:00"));
    assert_eq!(text(&session), source);
    argument(&mut session, "0");
    assert!(session.echo().contains("observed"));
    erase(&mut session, 1);
    argument(&mut session, "2");
    undo(&mut session, source, "<2024-10-27 Sun 02:30>* H\n");
    session.execute(Command::History(History::Undo));
    action(&mut session, C::Timestamp);
    argument(&mut session, "2024-03-31 02:30");
    assert!(session.echo().contains("Nonexistent"), "{}", session.echo());
    assert_eq!(text(&session), source);
    session.handle(control('g'));
    assert_eq!(text(&session), source);
}

// Calendar month/year movement clamps to the last valid day. Timestamp civil
// arithmetic's excess-day normalization must not make the picker skip a month.
#[test]
fn calendar_month_ends() {
    for (name, draft, command, expected) in [
        (
            "leap month",
            "2024-01-31",
            C::CalendarForwardMonth,
            "<2024-02-29 Thu>* H\n",
        ),
        (
            "leap year",
            "2024-02-29",
            C::CalendarForwardYear,
            "<2025-02-28 Fri>* H\n",
        ),
        (
            "year rollover",
            "2024-12-31",
            C::CalendarForwardMonth,
            "<2025-01-31 Fri>* H\n",
        ),
    ] {
        let (_fixture, session) = editor("", "notes.org", "* H\n");
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::Timestamp);
        session.handle(Event::Paste(draft.into()));
        session.execute(Command::Org(command));
        session.handle(key(KeyCode::Enter));
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
    }
}
