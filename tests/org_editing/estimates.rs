use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand};

// Default chord, M-x and a configured alias publish the same narrow cookie edit,
// preserving task state, tags, planning, CRLF/Unicode and final-newline spelling.
#[test]
fn priority_paths() {
    for path in ["key", "named", "alias"] {
        let source = "* TODO 界 :work:\r\nSCHEDULED: <2026-10-29 Thu>\r\nbody";
        let (_fixture, mut session) =
            editor("[keys.org]\n'M-j'='org-priority'", "notes.org", source);
        match path {
            "key" => {
                session.handle(control('c'));
                session.handle(key(KeyCode::Char(',')));
            }
            "named" => named(&mut session, "org-priority"),
            _ => {
                session.handle(meta('j'));
            }
        }
        settle(&mut session);
        argument(&mut session, "A");
        assert_eq!(
            text(&session),
            source.replace("* TODO ", "* TODO [#A] "),
            "{path}: {}",
            session.echo()
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
}

// Priority cycling follows the reference direction, including the no-cookie
// state; a missing cookie inspects as the configured default without editing.
#[test]
fn priority_cycle() {
    let source = "* Heading\n";
    for (command, expected) in [
        (OrgCommand::PriorityUp, ["C", "B", "A"]),
        (OrgCommand::PriorityDown, ["A", "B", "C"]),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        action(&mut session, OrgCommand::InspectPriority);
        assert_eq!(session.echo(), "Org priority: B (default)");
        for letter in expected {
            action(&mut session, command);
            assert_eq!(text(&session), format!("* [#{letter}] Heading\n"));
        }
        action(&mut session, command);
        assert_eq!(text(&session), source);
    }
}

// File-local ranges override only priority data, and clearing an explicit cookie
// retains the title. Malformed range declarations refuse a prospective edit.
#[test]
fn priority_declarations() {
    for (declaration, valid) in [
        ("D F E", true),
        ("F D E", false),
        ("D F", false),
        ("1 3 2", false),
    ] {
        let source = format!("#+PRIORITIES: {declaration}\n* TODO Title\n");
        let (_fixture, mut session) = editor("", "notes.org", &source);
        goto(&mut session, 2);
        action(&mut session, OrgCommand::PriorityDown);
        if valid {
            assert_eq!(
                text(&session),
                source.replace("TODO Title", "TODO [#D] Title")
            );
            action(&mut session, OrgCommand::Priority);
            argument(&mut session, " ");
            assert_eq!(text(&session), source);
        } else {
            assert_eq!(text(&session), source);
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}

// Effort entry validates elapsed duration forms but persists exactly the entered
// spelling. Drawer creation/replacement/removal is one normal history operation.
#[test]
fn effort_forms() {
    for (draft, minutes) in [
        ("30", 30.0),
        ("3:12", 192.0),
        ("1:23:45", 83.75),
        ("1d3h5min", 1625.0),
        ("1.5h", 90.0),
        ("1w2d", 12960.0),
        ("0.25min", 0.25),
    ] {
        let source = "* TODO [#A] Task :work:\r\nCLOSED: [2026-10-28 Wed 17:05]\r\n:PROPERTIES:\r\n:UNKNOWN: retain\r\n:END:\r\nbody";
        let (_fixture, mut session) = editor("", "notes.org", source);
        action(&mut session, OrgCommand::SetEffort);
        argument(&mut session, draft);
        let expected = source.replace(":END:", &format!(":EFFORT: {draft}\r\n:END:"));
        assert_eq!(text(&session), expected, "{draft}: {}", session.echo());
        action(&mut session, OrgCommand::InspectEffort);
        assert_eq!(
            session.echo(),
            format!("Org effort: {draft} = {minutes} minutes")
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), expected);
    }
}

// Allowed effort choices are source data with explicit inherited ownership;
// completion edits only the prompt, and cycling retains chosen source spelling.
#[test]
fn effort_choices() {
    let source = "#+PROPERTY: EFFORT_ALL 0 0:10 1h\n* Parent\n** TODO Child\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 3);
    action(&mut session, OrgCommand::SetEffort);
    session.handle(Event::Paste("1".into()));
    session.handle(key(KeyCode::Tab));
    session.handle(key(KeyCode::Enter));
    assert!(text(&session).contains(":EFFORT: 1h\n"));
    action(&mut session, OrgCommand::IncEffort);
    assert!(text(&session).contains(":EFFORT: 0\n"));
    action(&mut session, OrgCommand::IncEffort);
    assert!(text(&session).contains(":EFFORT: 0:10\n"));
    assert!(text(&session).starts_with("#+PROPERTY: EFFORT_ALL 0 0:10 1h\n* Parent\n"));
}
