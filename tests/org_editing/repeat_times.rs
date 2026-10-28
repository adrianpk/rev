use super::properties::argument;
use super::timestamps::dst_clock;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand as C};

// A next occurrence in a fold requires an observed offset. No keyword, log,
// property or planning edit appears before the complete offset selection.
#[test]
fn repeat_offset_choice() {
    let source = "* TODO H\nSCHEDULED: <2024-10-26 Sat 02:30 +1d>\n";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(dst_clock);
    action(&mut session, C::NextTodo);
    assert!(session.echo().contains("Ambiguous"), "{}", session.echo());
    assert_eq!(text(&session), source);
    argument(&mut session, "2");
    assert!(
        text(&session).contains("SCHEDULED: <2024-10-27 Sun 02:30 +1d>"),
        "{}",
        session.echo()
    );
    assert!(text(&session).contains("- State \"DONE\" from \"TODO\""));
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// A missing local time requests valid source data. Removing a clock or repeat
// cookie cannot bypass resolution; cancellation preserves the entire transition.
#[test]
fn repeat_gap_repair() {
    let source = "* TODO H\nSCHEDULED: <2024-03-30 Sat 02:30 +1d>\n";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(dst_clock);
    action(&mut session, C::NextTodo);
    assert!(session.echo().contains("Nonexistent"), "{}", session.echo());
    argument(&mut session, "[2024-03-31 Sun]");
    assert!(session.echo().contains("Retain"), "{}", session.echo());
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    session.handle(control('g'));
    action(&mut session, C::NextTodo);
    argument(&mut session, "2024-03-31 03:30");
    assert!(
        text(&session).contains("<2024-03-31 Sun 03:30 +1d>"),
        "{}",
        session.echo()
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// Every deferred stage retains its original revision. A canonical edit while
// choosing an offset or note revokes the complete repeat publication authority.
#[test]
fn repeat_stale_stages() {
    for (name, config, stamp) in [
        ("offset", "", "2024-10-26 Sat 02:30 +1d"),
        ("note", "org.todo.log_repeat='note'", "2024-02-28 Wed +1d"),
    ] {
        let source = format!("* TODO H\nSCHEDULED: <{stamp}>\n");
        let (_fixture, session) = editor(config, "notes.org", &source);
        let mut session = session.with_org_clock(dst_clock);
        action(&mut session, C::NextTodo);
        session.replace(0..0, "external ").unwrap();
        session.handle(key(KeyCode::Enter));
        assert!(
            session.echo().contains("source or view changed"),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), format!("external {source}"));
        assert_eq!(session.buffer().unwrap().history().len(), 1);
    }
}

// File repeat logging overrides the factory, and inherited/local LOGGING can
// explicitly turn it off again. No disabled history or LAST_REPEAT is invented.
#[test]
fn repeat_logging_overrides() {
    for (name, prefix, property, logged) in [
        ("file off", "#+STARTUP: nologrepeat\n", "", false),
        ("file on", "#+STARTUP: logrepeat\n", "", true),
        (
            "local off",
            "#+STARTUP: logrepeat\n",
            ":PROPERTIES:\n:LOGGING: nil\n:END:\n",
            false,
        ),
    ] {
        let source = format!("{prefix}* TODO H\nSCHEDULED: <2024-02-28 Wed +1d>\n{property}");
        let (_fixture, session) = editor("", "notes.org", &source);
        let mut session = session.with_org_clock(super::repeaters::clock);
        super::headings::goto(&mut session, 2);
        action(&mut session, C::NextTodo);
        assert!(
            text(&session).contains("<2024-02-29 Thu +1d>"),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session).contains(":LAST_REPEAT:"), logged, "{name}");
        assert_eq!(
            text(&session).contains("- State \"DONE\""),
            logged,
            "{name}"
        );
    }
}
