use rev::command::{History, Motion, OrgCommand};

use super::visibility::{OUTLINE, action, visible};
use super::*;

pub(super) fn goto(session: &mut Session, line: usize) {
    session.execute(Command::GotoLine);
    session.handle(Event::Paste(line.to_string()));
    session.handle(key(KeyCode::Enter));
}

pub(super) fn row(session: &Session) -> usize {
    session.view().cell(session.buffer().unwrap()).unwrap().row
}

// Native heading motion uses real source targets, stops at sibling boundaries,
// skips hidden descendants and leaves source/history and peer choices untouched.
#[test]
fn heading_motion() {
    let (_fixture, mut session) = editor("", "notes.org", OUTLINE);
    for (command, expected) in [
        (OrgCommand::NextHeading, 2),
        (OrgCommand::NextHeading, 4),
        (OrgCommand::ParentHeading, 2),
        (OrgCommand::ForwardHeading, 6),
        (OrgCommand::BackwardHeading, 2),
        (OrgCommand::PreviousHeading, 0),
    ] {
        action(&mut session, command);
        assert_eq!(row(&session), expected, "{command:?}: {}", session.echo());
    }
    action(&mut session, OrgCommand::Overview);
    let folded = visible(&session);
    action(&mut session, OrgCommand::NextHeading);
    assert_eq!(row(&session), 8);
    action(&mut session, OrgCommand::PreviousHeading);
    assert_eq!(row(&session), 0);
    assert_eq!(visible(&session), folded);
    goto(&mut session, 5);
    let before = session.view().clone();
    action(&mut session, OrgCommand::ForwardHeading);
    assert!(session.echo().contains("No same-level"));
    assert_eq!(session.view(), &before);
    assert_eq!(text(&session), OUTLINE);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Insertion follows the default heading split at point, retaining every existing
// section as source. Explicit initiating point and one undo/redo cover publication.
#[test]
fn insert_heading() {
    for (name, source, point, expected) in [
        (
            "title end",
            "* Root\nbody\n** Child\nx\n",
            6,
            "* Root\n* \nbody\n** Child\nx\n",
        ),
        ("title middle", "** Root\nbody", 5, "** Ro\n** ot\nbody"),
        ("heading start", "* Root\nbody", 0, "* \n* Root\nbody"),
        ("paragraph", "* Root\nbody", 9, "* Root\nbo\n* dy"),
        ("preamble", "text\n* Root\n", 0, "* text\n* Root\n"),
        ("empty", "", 0, "* "),
        ("terminated EOF", "* Root\n", 7, "* Root\n* \n"),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        for _ in 0..point {
            session.execute(Command::Motion(Motion::ForwardChar));
        }
        let before = session.view().point.position;
        action(&mut session, OrgCommand::InsertHeading);
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source, "{name}");
        assert_eq!(session.view().point.position, before, "{name}");
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), expected, "{name}");
    }
}

// Literal block headings never become navigation candidates, and incomplete
// observations cannot certify a motion or a structural source edit.
#[test]
fn observed_targets() {
    let source = "* Root\n#+begin_src text\n,* Fake\n#+end_src\n* Peer\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    action(&mut session, OrgCommand::NextHeading);
    assert_eq!(row(&session), 4);
    let source = "* Entry\n".repeat(4097);
    let (_fixture, mut session) = editor("", "notes.org", &source);
    for command in [OrgCommand::NextHeading, OrgCommand::InsertHeading] {
        let before = session.view().clone();
        action(&mut session, command);
        assert!(session.echo().contains("incomplete"));
        assert_eq!(session.view(), &before);
        assert_eq!(text(&session), source);
    }
}
