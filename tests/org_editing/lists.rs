use super::headings::goto;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand};

// Continuation splits actual item text, carries the marker/description shape and
// updates ordered siblings as one undoable edit, including counter resets.
#[test]
fn item_insertion() {
    for (name, source, column, expected) in [
        (
            "unordered",
            "- first tail\n- next\n",
            7,
            "- first\n-  tail\n- next\n",
        ),
        (
            "ordered",
            "1. first\n2. next\n",
            8,
            "1. first\n2. \n3. next\n",
        ),
        (
            "reset",
            "3. [@20] first\n4. next\n",
            14,
            "20. [@20] first\n21. \n22. next\n",
        ),
        (
            "description",
            "- term :: value\n",
            0,
            "-  :: \n- term :: value\n",
        ),
        ("checkbox", "- [X] done\n", 10, "- [X] done\n- [ ] \n"),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        for _ in 0..column {
            session.execute(Command::Motion(rev::command::Motion::ForwardChar));
        }
        action(&mut session, OrgCommand::MetaReturn);
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source, "{name}");
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), expected, "{name}");
    }
}

// Moves carry all continuation and nested source. Renumbering belongs to that
// same history operation and preserves the explicit reset and marker terminator.
#[test]
fn item_moves() {
    let source = "* Root\n1) first\n   paragraph\n   - child\n     #+begin_example\n     - literal\n     #+end_example\n2) second\n3) [@20] third\n";
    let expected = "* Root\n1) second\n2) first\n   paragraph\n   - child\n     #+begin_example\n     - literal\n     #+end_example\n20) [@20] third\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 2);
    action(&mut session, OrgCommand::MetaDown);
    assert_eq!(text(&session), expected, "{}", session.echo());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
}

// Ordinary indentation leaves children at their existing level. Tree commands
// carry nested content, and ordinary outdent refuses to detach children.
#[test]
fn item_indentation() {
    let source = "- first\n- second\n  - child\n- last\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 2);
    action(&mut session, OrgCommand::MetaRight);
    assert_eq!(
        text(&session),
        "- first\n  - second\n  - child\n- last\n",
        "{}",
        session.echo()
    );
    session.execute(Command::History(History::Undo));
    action(&mut session, OrgCommand::IndentItemTree);
    let indented = "- first\n  - second\n    - child\n- last\n";
    assert_eq!(text(&session), indented);
    action(&mut session, OrgCommand::MetaLeft);
    assert_eq!(text(&session), indented);
    assert!(session.echo().contains("without its children"));
    action(&mut session, OrgCommand::OutdentItemTree);
    assert_eq!(text(&session), source);
}

// Unchanged source anchors survive a counter-width change and original-fragment
// reordering. CRLF/BOM/no-final-newline spelling survives save and full replay.
#[test]
fn item_width_eof() {
    let source = "\u{feff}8. [@8] first\r\n9. second\r\n10. third";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 2);
    action(&mut session, OrgCommand::MetaDown);
    let expected = "8. [@8] first\r\n9. third\r\n10. second";
    assert_eq!(text(&session), expected, "{}", session.echo());
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source.trim_start_matches('\u{feff}'));
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
}
