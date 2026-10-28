use super::headings::goto;
use super::properties::argument;
use super::repeaters::clock;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand as C};
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};

fn early(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    ZonedOperation::capture(
        &ZoneSelection::Utc,
        &ZoneEnvironment {
            localtime: "unused".into(),
            zoneinfo: "unused".into(),
            tz: None,
        },
        || Ok(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1709263800)),
    )
}
// Before rollover, date-only restart/catch-up use the previous Org day. Stored
// backlog and timed repeats keep their own semantics; real log time is unchanged.
#[test]
fn habit_early_completion() {
    for (name, boundary, cookie, timed, expected) in [
        ("midnight", 0, ".+1d", false, "2024-03-02 Sat"),
        ("extended restart", 4, ".+1d", false, "2024-03-01 Fri"),
        ("extended catchup", 4, "++1d", false, "2024-03-01 Fri"),
        ("backlog", 4, "+1d", false, "2024-03-01 Fri"),
        ("timed restart", 4, ".+1d", true, "2024-03-02 Sat"),
        ("timed catchup", 4, "++1d", true, "2024-03-01 Fri"),
    ] {
        let clock = if timed { " 14:00" } else { "" };
        let source = format!(
            "* TODO Habit\nSCHEDULED: <2024-02-29 Thu{clock} {cookie}>\n:PROPERTIES:\n:STYLE: habit\n:END:\n"
        );
        let (_fixture, session) = editor(
            &format!("org.time.extend_today_until={boundary}"),
            "notes.org",
            &source,
        );
        let mut session = session.with_org_clock(early);
        action(&mut session, C::NextTodo);
        let result = text(&session);
        assert!(
            result.contains(&format!("<{expected}{clock} {cookie}>")),
            "{name}: {} {result}",
            session.echo()
        );
        assert!(
            result.contains(":LAST_REPEAT: [2024-03-01 Fri 03:30]"),
            "{name}"
        );
        assert!(result.contains("- State \"DONE\" from \"TODO\" [2024-03-01 Fri 03:30]"));
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
}

// File-local keywords and cancellable note policy retain exact old logs. The
// completed transition saves BOM/CRLF and maps a peer view, then undoes as one.
#[test]
fn habit_logged_save() {
    let source = "#+SEQ_TODO: OPEN WAIT | FINISHED\r\n#+STARTUP: lognoterepeat\r\n* WAIT Habit\r\nSCHEDULED: <2024-02-28 Wed .+2d/3d>\r\n:PROPERTIES:\r\n:STYLE: habit\r\n:LOG_INTO_DRAWER: LOGBOOK\r\n:LAST_REPEAT: [2024-02-26 Mon 23:00]\r\n:END:\r\n:LOGBOOK:\r\n- State \"FINISHED\"  from \"WAIT\"  [2024-02-26 Mon 23:00] \\\\\r\n  old note 界\r\n:END:\r\n* Peer\r\nmarker";
    let (fixture, session) = editor("", "notes.org", &format!("\u{feff}{source}"));
    let mut session = session.with_org_clock(clock);
    goto(&mut session, 15);
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Window(WindowCommand::Other));
    let peer = session
        .windows()
        .order()
        .iter()
        .copied()
        .find(|id| *id != session.windows().selected_id())
        .unwrap();
    let anchor = session.windows().get(peer).unwrap().point.position;
    goto(&mut session, 3);
    action(&mut session, C::NextTodo);
    assert_eq!(text(&session), source);
    session.handle(control('g'));
    assert!(session.buffer().unwrap().history().is_empty());
    action(&mut session, C::NextTodo);
    argument(&mut session, "new note");
    let result = text(&session);
    assert!(
        result.contains("* OPEN Habit\r\nSCHEDULED: <2024-03-03 Sun .+2d/3d>"),
        "{}",
        session.echo()
    );
    assert!(result.contains("- State \"FINISHED\" from \"WAIT\" [2024-03-01 Fri 14:30]"));
    assert!(result.contains(
        "- State \"FINISHED\"  from \"WAIT\"  [2024-02-26 Mon 23:00] \\\\\r\n  old note 界\r\n"
    ));
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        result[..result.find("marker").unwrap()].chars().count()
    );
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(fixture.0.join("notes.org")).unwrap(),
        format!("\u{feff}{result}").as_bytes()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(session.windows().get(peer).unwrap().point.position, anchor);
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(fixture.0.join("notes.org")).unwrap(),
        format!("\u{feff}{source}").as_bytes()
    );
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), result);
}

// Source-local repeat policy can suppress new history without erasing existing
// observations or fabricating completion dates from an old LAST_REPEAT property.
#[test]
fn habit_logging_overrides() {
    for (name, prefix, logging, logged) in [
        ("file off", "#+STARTUP: nologrepeat\n", "", false),
        (
            "local off",
            "#+STARTUP: logrepeat\n",
            ":LOGGING: nil\n",
            false,
        ),
        (
            "local on",
            "#+STARTUP: nologrepeat\n",
            ":LOGGING: logrepeat\n",
            true,
        ),
    ] {
        let old = "- State \"DONE\" from \"TODO\" [2024-02-26 Mon]\n";
        let source = format!(
            "{prefix}* TODO Habit\nSCHEDULED: <2024-02-28 Wed .+2d/3d>\n:PROPERTIES:\n:STYLE: habit\n{logging}:LAST_REPEAT: [2024-02-26 Mon]\n:END:\n{old}"
        );
        let (_fixture, session) = editor("", "notes.org", &source);
        let mut session = session.with_org_clock(clock);
        goto(&mut session, 2);
        action(&mut session, C::NextTodo);
        let result = text(&session);
        assert!(result.contains(old), "{name}");
        assert_eq!(
            result.matches("- State \"DONE\"").count(),
            if logged { 2 } else { 1 },
            "{name}"
        );
        assert!(result.contains(if logged {
            ":LAST_REPEAT: [2024-03-01 Fri 14:30]"
        } else {
            ":LAST_REPEAT: [2024-02-26 Mon]"
        }));
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
}

fn denied(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    Err("habit clock must not be captured for read-only source".into())
}
// Read-only habit completion rejects before capturing time; the same guard also
// protects malformed habit metadata from an accidental ordinary-task fallback.
#[test]
fn habit_readonly() {
    let source =
        "* TODO Habit\nSCHEDULED: <2024-02-28 Wed .+2d/3d>\n:PROPERTIES:\n:STYLE: habit\n:END:\n";
    let (mut buffers, id) = super::folds::buffer(source);
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(true);
    let view = View::new(buffer);
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
    .with_org_clock(denied);
    action(&mut session, C::NextTodo);
    assert!(session.echo().contains("read-only"), "{}", session.echo());
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}
