use rev::command::{History, OrgCommand};

use super::headings::{goto, row};
use super::visibility::action;
use super::*;

pub(super) const FIRST: &str = "** First\n:PROPERTIES:\n:ID: first\n:END:\nbody\n\n*** Child\n#+name: data\n| a | b |\n#+TBLFM: $1=2\n#+begin_example\n,* Literal\n#+end_example\n\n";
pub(super) const SECOND: &str = "** Second\nsecond body\n";
pub(super) fn source() -> String {
    format!("* Root\n{FIRST}{SECOND}* Peer\nlast\n")
}

// Level changes touch only the real heading prefixes, including descendants.
// Metadata, literal stars and table affiliations remain exact original source.
#[test]
fn subtree_levels() {
    let source = source();
    let (_fixture, mut session) = editor("", "notes.org", &source);
    goto(&mut session, 16);
    action(&mut session, OrgCommand::DemoteSubtree);
    assert_eq!(
        text(&session),
        source.replacen("** Second", "*** Second", 1)
    );
    action(&mut session, OrgCommand::PromoteSubtree);
    assert_eq!(text(&session), source);
    goto(&mut session, 2);
    action(&mut session, OrgCommand::PromoteSubtree);
    let promoted = source
        .replacen("** First", "* First", 1)
        .replacen("*** Child", "** Child", 1);
    assert_eq!(text(&session), promoted);
    assert_eq!(session.buffer().unwrap().history().len(), 3);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), promoted);
}

// Sibling moves carry the complete owned source and keep point on its heading.
// A single undo/redo reverses the whole operation, without detached metadata.
#[test]
fn subtree_moves() {
    let source = source();
    let (_fixture, mut session) = editor("", "notes.org", &source);
    goto(&mut session, 2);
    action(&mut session, OrgCommand::MoveSubtreeDown);
    let moved = format!("* Root\n{SECOND}{FIRST}* Peer\nlast\n");
    assert_eq!(text(&session), moved, "{}", session.echo());
    assert_eq!(row(&session), 3);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(row(&session), 1);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), moved);
    action(&mut session, OrgCommand::MoveSubtreeUp);
    assert_eq!(text(&session), source);
}

// An unterminated EOF exchanges its existing separator run with the sibling,
// retaining exact BOM, CRLF/LF bytes and the original final-newline state.
#[test]
fn subtree_eof() {
    for newline in ["\n", "\r\n"] {
        for gap in [1, 3] {
            let source = format!("\u{feff}* 界{newline}body{}* Last", newline.repeat(gap));
            let (fixture, mut session) = editor("", "notes.org", &source);
            action(&mut session, OrgCommand::MoveSubtreeDown);
            let expected = format!("* Last{}* 界{newline}body", newline.repeat(gap));
            assert_eq!(text(&session), expected, "{}", session.echo());
            session.execute(Command::SaveBuffer);
            assert_eq!(
                std::fs::read(fixture.0.join("notes.org")).unwrap(),
                format!("\u{feff}{expected}").as_bytes()
            );
            session.execute(Command::History(History::Undo));
            assert_eq!(text(&session), source.trim_start_matches('\u{feff}'));
            session.execute(Command::History(History::Redo));
            assert_eq!(text(&session), expected);
        }
    }
}
