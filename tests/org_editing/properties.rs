use super::headings::goto;
use super::visibility::{action, visible};
use super::*;
use rev::command::{History, OrgCommand};

pub(super) fn argument(session: &mut Session, value: &str) {
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

// Only selected names inherit; bases override ancestors and local additions
// retain all source contributors. Neither inspection nor inheritance inserts text.
#[test]
fn selective_properties() {
    let source = "#+PROPERTY: X file\n#+PROPERTY: X+ default\n* Parent\n:PROPERTIES:\n:X: parent\n:Y: secret\n:END:\n** Child\n:PROPERTIES:\n:X+: child\n:END:\nbody\n";
    for (name, config, property, value, origin) in [
        ("factory", "", "X", "child", "local X+ at line 10"),
        (
            "selected",
            "[org.properties]\ninherit = ['x']",
            "X",
            "parent child",
            "inherited X at line 5",
        ),
        (
            "excluded",
            "[org.properties]\ninherit = ['X']",
            "Y",
            "",
            "absent",
        ),
    ] {
        let (_fixture, mut session) = editor(config, "notes.org", source);
        goto(&mut session, 8);
        action(&mut session, OrgCommand::InspectProperty);
        argument(&mut session, property);
        assert!(
            session.echo().contains(&format!("= {value:?}")),
            "{name}: {}",
            session.echo()
        );
        assert!(
            session.echo().contains(origin),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), source, "{name}");
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// File declarations are data defaults, superseded by drawer/nearest local bases.
// Additions before a local base still extend it; nil stops the ancestor search.
#[test]
fn property_precedence() {
    for (name, source, row, expected, origin) in [
        (
            "file",
            "#+PROPERTY: X file\n#+PROPERTY: X+ extra\n* H\n",
            3,
            "file extra",
            "file directive X at line 1",
        ),
        (
            "reset",
            "#+PROPERTY: X old\n#+PROPERTY: X+ gone\n#+PROPERTY: X fresh\n* H\n",
            4,
            "fresh",
            "file directive X at line 3",
        ),
        (
            "drawer",
            ":PROPERTIES:\n:X: drawer\n:END:\n#+PROPERTY: X file\n* H\n",
            5,
            "drawer",
            "file drawer X at line 2",
        ),
        (
            "local",
            "#+PROPERTY: X file\n* H\n:PROPERTIES:\n:X+: extra\n:X: local\n:END:\n",
            2,
            "local extra",
            "local X at line 5",
        ),
        (
            "nil",
            "* P\n:PROPERTIES:\n:X: parent\n:END:\n** C\n:PROPERTIES:\n:X: nil\n:END:\n*** Grand\n",
            9,
            "<undefined>",
            "inherited X at line 7",
        ),
    ] {
        let (_fixture, mut session) =
            editor("[org.properties]\ninherit = ['X']", "notes.org", source);
        goto(&mut session, row);
        action(&mut session, OrgCommand::InspectProperty);
        argument(&mut session, "x");
        assert!(
            session.echo().contains(&format!("= {expected:?}")),
            "{name}: {}",
            session.echo()
        );
        assert!(
            session.echo().contains(origin),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), source, "{name}");
    }
}

// Ordinary local edits shadow inherited data. The explicit owner action asks
// which contribution to edit and rewrites exactly that source value atomically.
#[test]
fn actual_property_owner() {
    let source =
        "* Parent\n:PROPERTIES:\n:X: parent\n:END:\n** Child\n:PROPERTIES:\n:X+: child\n:END:\n";
    for (number, before, after) in [
        ("1", ":X: parent", ":X: changed"),
        ("2", ":X+: child", ":X+: changed"),
    ] {
        let (_fixture, mut session) = editor("org.properties.inherit = ['X']", "notes.org", source);
        goto(&mut session, 5);
        action(&mut session, OrgCommand::EditPropertyOwner);
        argument(&mut session, "X");
        assert!(session.echo().contains("inherited X at line 3"));
        assert!(session.echo().contains("local X+ at line 7"));
        argument(&mut session, number);
        assert!(
            session
                .echo()
                .contains(if number == "1" { "line 3" } else { "line 7" })
        );
        argument(&mut session, "changed");
        assert_eq!(text(&session), source.replace(before, after));
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
        assert!(!session.buffer().unwrap().is_modified());
    }
    let (_fixture, mut session) = editor("org.properties.inherit = ['X']", "notes.org", source);
    goto(&mut session, 5);
    set(&mut session, "X", "local");
    assert_eq!(
        text(&session),
        source.replace(":X+: child\n", ":X+: child\n:X: local\n")
    );
    action(&mut session, OrgCommand::DeleteProperty);
    argument(&mut session, "X");
    assert_eq!(text(&session), source);
}

// Explicit inherited-owner edits can target file directives without creating a
// child drawer, and undo restores shared cursors against the canonical source.
#[test]
fn file_property_owner() {
    let source = "#+PROPERTY: X default\n* Head\nbody\n* Peer\nmarker\n";
    let (_fixture, mut session) = editor("org.properties.inherit = ['X']", "notes.org", source);
    goto(&mut session, 5);
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Window(WindowCommand::Other));
    let peer = session
        .windows()
        .order()
        .iter()
        .copied()
        .find(|id| *id != session.windows().selected_id())
        .unwrap();
    goto(&mut session, 2);
    action(&mut session, OrgCommand::EditPropertyOwner);
    argument(&mut session, "X");
    assert!(session.echo().contains("file directive X at line 1"));
    argument(&mut session, "longer value 界");
    let expected = source.replace("default", "longer value 界");
    assert_eq!(text(&session), expected);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        expected[..expected.find("marker").unwrap()].chars().count()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        source.find("marker").unwrap()
    );
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
}

// A draft is never authority after its source revision changes. Prompt-owned
// keys cannot move to another window; cancel does not retain an active draft.
#[test]
fn property_draft_source() {
    let source = "* Head\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    session.execute(Command::Split(Split::Right));
    let other = session
        .windows()
        .order()
        .iter()
        .copied()
        .find(|id| *id != session.windows().selected_id())
        .unwrap();
    action(&mut session, OrgCommand::SetProperty);
    argument(&mut session, "X");
    assert!(session.select_view(other).is_err());
    session.replace(0..0, "prefix\n").unwrap();
    argument(&mut session, "stale");
    assert!(session.echo().contains("source or view changed"));
    assert_eq!(text(&session), "prefix\n* Head\n");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    action(&mut session, OrgCommand::SetProperty);
    argument(&mut session, "X");
    session.execute(Command::KeyboardQuit);
    session.handle(Event::Paste("ordinary".into()));
    assert_eq!(text(&session), "ordinary* Head\n");
}
