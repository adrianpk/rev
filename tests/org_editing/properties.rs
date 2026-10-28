use super::headings::goto;
use super::visibility::{action, visible};
use super::*;
use rev::command::{History, OrgCommand};

fn argument(session: &mut Session, value: &str) {
    session.handle(Event::Paste(value.into()));
    session.handle(key(KeyCode::Enter));
}
fn set(session: &mut Session, name: &str, value: &str) {
    action(session, OrgCommand::SetProperty);
    argument(session, name);
    argument(session, value);
}

// New drawers follow the heading and planning metadata. Existing value edits
// preserve exact delimiters, unknown fields, CRLF, UTF-8 and empty EOF spelling.
#[test]
fn local_properties() {
    for (name, source, row, expected) in [
        (
            "new",
            "* Head\nbody\n",
            1,
            "* Head\n:PROPERTIES:\n:Custom: café\n:END:\nbody\n",
        ),
        (
            "planning",
            "* Head\nSCHEDULED: <2026-10-28 Wed>\nbody\n",
            2,
            "* Head\nSCHEDULED: <2026-10-28 Wed>\n:PROPERTIES:\n:Custom: café\n:END:\nbody\n",
        ),
        (
            "empty eof",
            "* Head",
            1,
            "* Head\n:PROPERTIES:\n:Custom: café\n:END:",
        ),
        (
            "existing",
            "* Head\r\n  :PROPERTIES:\r\n  :Custom:   old  \r\n  :Other: unchanged\r\n  :END:\r\n",
            4,
            "* Head\r\n  :PROPERTIES:\r\n  :Custom:   café  \r\n  :Other: unchanged\r\n  :END:\r\n",
        ),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, row);
        set(&mut session, "Custom", "café");
        // Property identity is case-insensitive; new canonical keys use uppercase.
        let expected = expected.replace(":Custom: café", ":CUSTOM: café");
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source, "{name}");
        assert!(!session.buffer().unwrap().is_modified(), "{name}");
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), expected, "{name}");
    }
}

// Additive spelling is retained as source; removal affects only the requested
// local base or + form, and ordinary drawers never become property owners.
#[test]
fn additive_and_drawers() {
    let source = "* H\n:PROPERTIES:\n:X: base\n:X+: extra\n:UNKNOWN: keep\n:END:\n:LOGBOOK:\n:X: ordinary\n:END:\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    set(&mut session, "x+", "tail");
    assert!(text(&session).contains(":X+: extra\n:UNKNOWN: keep\n:X+: tail\n:END:"));
    action(&mut session, OrgCommand::InspectProperty);
    argument(&mut session, "x");
    assert!(
        session.echo().contains("base extra tail"),
        "{}",
        session.echo()
    );
    action(&mut session, OrgCommand::DeleteProperty);
    argument(&mut session, "x+");
    assert_eq!(text(&session), source.replace(":X+: extra\n", ""));
    goto(&mut session, 2);
    action(&mut session, OrgCommand::HideDrawer);
    assert_eq!(visible(&session), [0, 1, 4, 5, 6, 7, 8]);
    action(&mut session, OrgCommand::ShowDrawer);
    goto(&mut session, 6);
    let before = text(&session);
    action(&mut session, OrgCommand::SetProperty);
    assert!(session.echo().contains("unavailable"), "{}", session.echo());
    assert_eq!(text(&session), before);
}

// Each prompt stage belongs to its original qualified view. Cancellation,
// literal forms, invalid delimiters and derived fields leave source/history alone.
#[test]
fn property_admission() {
    for (name, source, row, property) in [
        ("delimiter", "* H\n", 1, "END"),
        ("derived", "* H\n", 1, "ITEM"),
        (
            "literal",
            "* H\n#+begin_example\n:X: text\n#+end_example\n",
            3,
            "X",
        ),
        ("malformed", "* H\n:PROPERTIES:\n:X: value\n", 2, "X"),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, row);
        action(&mut session, OrgCommand::SetProperty);
        if matches!(name, "delimiter" | "derived") {
            argument(&mut session, property);
        }
        session.execute(Command::KeyboardQuit);
        assert_eq!(text(&session), source, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
    let (_fixture, mut session) = editor("", "notes.org", "* H\n");
    action(&mut session, OrgCommand::SetProperty);
    argument(&mut session, "X");
    session.execute(Command::KeyboardQuit);
    assert_eq!(text(&session), "* H\n");
    assert!(session.buffer().unwrap().history().is_empty());
}
