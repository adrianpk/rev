use rev::command::{History, Motion, OrgCommand, RegionCommand};
use rev::coordinates::Scalar;
use rev::window::ViewId;

use super::headings::{goto, row};
use super::subtrees::{FIRST, SECOND, source};
use super::visibility::{action, visible};
use super::*;

fn excerpt(session: &Session, position: Scalar) -> String {
    let buffer = session.buffer().unwrap();
    let range = position.0..(position.0 + 32).min(buffer.len_chars());
    buffer
        .slice(&buffer.span(range).unwrap())
        .unwrap()
        .to_string()
}

fn at_view(session: &Session, id: ViewId) -> String {
    excerpt(session, session.windows().get(id).unwrap().point.position)
}

fn top_line(session: &Session, id: ViewId) -> String {
    let buffer = session.buffer().unwrap();
    let coordinates = rev::coordinates::Coordinates::new(buffer).unwrap();
    let row = session.windows().get(id).unwrap().top;
    let start = coordinates
        .from_line(rev::coordinates::LineByte { row, column: 0 })
        .unwrap();
    let end = coordinates.line_end(start).unwrap();
    buffer
        .slice(&buffer.span(start.0..end.0).unwrap())
        .unwrap()
        .to_string()
}

pub(super) fn visit_text(session: &mut Session, target: &str) {
    let source = text(session);
    let byte = source.find(target).unwrap();
    let line = source[..byte].bytes().filter(|b| *b == b'\n').count() + 1;
    goto(session, line);
}

fn keep_mark(session: &mut Session) {
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::KeyboardQuit);
}

// Marks, independent points and viewport anchors follow their original source
// fragments. History maps a peer's current position, not its historical snapshot.
#[test]
fn subtree_peers() {
    let source = source();
    let (_fixture, mut session) = editor("", "notes.org", &source);
    visit_text(&mut session, "body\n\n*** Child");
    session.execute(Command::Motion(Motion::ForwardChar));
    keep_mark(&mut session);
    let mark = session.buffer().unwrap().mark().unwrap();
    session.execute(Command::Split(Split::Right));
    let origin = session.windows().selected_id();
    session.execute(Command::Window(WindowCommand::Other));
    let peer = session.windows().selected_id();
    visit_text(&mut session, "*** Child");
    session.execute(Command::Scroll(rev::command::Scroll::Recenter));
    let top_text = top_line(&session, peer);
    assert_ne!(top_text, "* Root");
    session.execute(Command::Window(WindowCommand::Other));
    goto(&mut session, 2);
    let before = session.view().point.position;
    action(&mut session, OrgCommand::MoveSubtreeDown);
    assert_eq!(
        text(&session),
        format!("* Root\n{SECOND}{FIRST}* Peer\nlast\n")
    );
    assert!(at_view(&session, peer).starts_with("*** Child\n"));
    assert!(
        excerpt(&session, session.buffer().unwrap().mark().unwrap())
            .starts_with("ody\n\n*** Child")
    );
    assert!(session.buffer().unwrap().is_modified());
    assert_eq!(top_line(&session, peer), top_text);
    session.execute(Command::Window(WindowCommand::Other));
    visit_text(&mut session, "| a | b |");
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(origin).unwrap().point.position,
        before
    );
    assert!(at_view(&session, peer).starts_with(" a | b |\n"));
    assert_eq!(session.buffer().unwrap().mark(), Some(mark));
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert!(at_view(&session, peer).starts_with(" a | b |\n"));
    assert!(at_view(&session, origin).starts_with("** First\n"));
    assert!(
        excerpt(&session, session.buffer().unwrap().mark().unwrap())
            .starts_with("ody\n\n*** Child")
    );
}

// EOF inside an unterminated last line belongs to that line's source fragment.
// The inverse also maps a peer moved to the new EOF after publication.
#[test]
fn subtree_eof_points() {
    let source = "* First\r\nbody\r\n\r\n* Last\r\nend";
    let (_fixture, mut session) = editor("", "notes.org", source);
    session.execute(Command::Motion(Motion::BufferEnd));
    keep_mark(&mut session);
    let end = session.view().point.position;
    session.execute(Command::Split(Split::Right));
    let origin = session.windows().selected_id();
    session.execute(Command::Window(WindowCommand::Other));
    let peer = session.windows().selected_id();
    goto(&mut session, 2);
    session.execute(Command::Window(WindowCommand::Other));
    action(&mut session, OrgCommand::MoveSubtreeUp);
    assert_eq!(text(&session), "* Last\r\nend\r\n\r\n* First\r\nbody");
    assert_eq!(row(&session), 1);
    assert_eq!(
        session
            .view()
            .cell(session.buffer().unwrap())
            .unwrap()
            .column,
        3
    );
    assert_eq!(
        session.buffer().unwrap().mark(),
        Some(session.view().point.position)
    );
    assert!(at_view(&session, peer).starts_with("body"));
    session.execute(Command::Window(WindowCommand::Other));
    session.execute(Command::Motion(Motion::BufferEnd));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(session.windows().get(origin).unwrap().point.position, end);
    assert!(at_view(&session, peer).starts_with("\r\n\r\n* Last"));
    session.execute(Command::History(History::Redo));
    assert_eq!(
        session.view().point.position.0,
        session.buffer().unwrap().len_chars()
    );
}

// Canonical level publication maps a mark through every preceding star edit,
// keeps metadata intact, and retires obsolete folds in all shared views.
#[test]
fn subtree_level_marks() {
    let source = source();
    let (_fixture, mut session) = editor("", "notes.org", &source);
    visit_text(&mut session, "| a | b |");
    session.execute(Command::Motion(Motion::ForwardChar));
    keep_mark(&mut session);
    let mark = session.buffer().unwrap().mark().unwrap();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    action(&mut session, OrgCommand::Overview);
    visit_text(&mut session, "** First");
    action(&mut session, OrgCommand::PromoteSubtree);
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(mark.0 - 2)));
    assert!(excerpt(&session, session.buffer().unwrap().mark().unwrap()).starts_with(" a | b |\n"));
    let buffer = session.buffer().unwrap();
    assert_eq!(
        session.windows().get(peer).unwrap().visible_lines(buffer),
        buffer.len_lines()
    );
    assert_eq!(visible(&session).len(), buffer.len_lines());
    session.execute(Command::History(History::Undo));
    assert_eq!(session.buffer().unwrap().mark(), Some(mark));
    assert_eq!(text(&session), source);
    session.execute(Command::History(History::Redo));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(mark.0 - 2)));
}
