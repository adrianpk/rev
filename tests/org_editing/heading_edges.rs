use rev::command::{History, Motion, OrgCommand};
use rev::coordinates::{Coordinates, Scalar};

use super::heading_context::meta_return;
use super::headings::{goto, row};
use super::visibility::{OUTLINE, action, visible};
use super::*;

// Read-only buffers admit real heading motion and hierarchy reveal but reject
// both insertion names without changing revision, source, cursor or history.
#[test]
fn readonly_headings() {
    let (mut buffers, id) = super::folds::buffer(OUTLINE);
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap();
    let stamp = session.buffer().unwrap().stamp();
    for (command, expected) in [
        (OrgCommand::NextHeading, 2),
        (OrgCommand::ForwardHeading, 6),
        (OrgCommand::BackwardHeading, 2),
        (OrgCommand::ParentHeading, 0),
        (OrgCommand::NextHeading, 2),
        (OrgCommand::PreviousHeading, 0),
    ] {
        action(&mut session, command);
        assert_eq!(row(&session), expected, "{command:?}");
    }
    for command in [OrgCommand::MetaReturn, OrgCommand::InsertHeading] {
        let before = session.view().clone();
        action(&mut session, command);
        assert!(session.echo().contains("read-only"));
        assert_eq!(session.view(), &before);
    }
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&session), OUTLINE);
    assert!(!session.buffer().unwrap().is_modified());
    assert!(session.buffer().unwrap().history().is_empty());
}

// Native Org keys and direct entry cannot escape inactive mode or a prompt.
// Canceling a prefix/query consumes only that interaction, never inserting a heading.
#[test]
fn context_ownership() {
    let (_fixture, mut session) = editor("", "notes.txt", OUTLINE);
    for command in [
        OrgCommand::MetaReturn,
        OrgCommand::InsertHeading,
        OrgCommand::NextHeading,
    ] {
        action(&mut session, command);
        assert!(session.echo().contains("not active"));
    }
    meta_return(&mut session);
    assert_eq!(text(&session), OUTLINE);
    session.execute(Command::Mode(Mode::Org));
    settle(&mut session);
    session.execute(Command::GotoLine);
    session.handle(Event::Paste("2".into()));
    meta_return(&mut session);
    session.execute(Command::Org(OrgCommand::MetaReturn));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|line| line.text().contains("Goto line: 2"))
    );
    session.handle(control('g'));
    session.handle(control('c'));
    session.handle(control('g'));
    session.handle(meta('x'));
    session.handle(Event::Paste("org-meta-return".into()));
    session.handle(control('g'));
    assert_eq!(text(&session), OUTLINE);
    assert_eq!(session.view().point.position, Scalar(0));
    assert!(session.buffer().unwrap().history().is_empty());
}

// Logical source bounds include offscreen headings, skipped numerical levels,
// Unicode titles, malformed star spellings and a final unterminated line.
#[test]
fn source_bounds() {
    let source = "preamble\n*Root text\n* 界e\u{301}\r\nbody\r\n*** Child\r\nchild\r\n* Peer";
    let (_fixture, mut session) = editor("", "notes.org", source);
    session.handle(Event::Resize(80, 3));
    action(&mut session, OrgCommand::NextHeading);
    assert_eq!(row(&session), 2);
    action(&mut session, OrgCommand::NextHeading);
    assert_eq!(row(&session), 4);
    action(&mut session, OrgCommand::ParentHeading);
    assert_eq!(row(&session), 2);
    action(&mut session, OrgCommand::ForwardHeading);
    assert_eq!(row(&session), 6);
    action(&mut session, OrgCommand::PreviousHeading);
    assert_eq!(row(&session), 4);
    assert!(
        Coordinates::new(session.buffer().unwrap())
            .unwrap()
            .is_boundary(session.view().point.position)
            .unwrap()
    );
    session.handle(Event::Resize(0, 0));
    action(&mut session, OrgCommand::ParentHeading);
    assert_eq!(row(&session), 2);
    for command in [
        OrgCommand::PreviousHeading,
        OrgCommand::ParentHeading,
        OrgCommand::BackwardHeading,
    ] {
        let before = session.view().clone();
        action(&mut session, command);
        assert!(session.echo().starts_with("No "));
        assert_eq!(session.view(), &before);
    }
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// One insertion maps peers from their current canonical points. Undo invoked
// from a peer restores the original initiator and retires folds in every view.
#[test]
fn shared_history() {
    let (_fixture, mut session) = editor("", "notes.org", OUTLINE);
    session.execute(Command::Split(Split::Right));
    let origin = session.windows().selected_id();
    session.execute(Command::Window(WindowCommand::Other));
    goto(&mut session, 9);
    let peer = session.windows().selected_id();
    action(&mut session, OrgCommand::Overview);
    session.execute(Command::Window(WindowCommand::Other));
    session.execute(Command::Motion(Motion::LineEnd));
    let before = session.view().point.position;
    meta_return(&mut session);
    let edited = text(&session);
    let inserted = session.view().point.position;
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    let buffer = session.buffer().unwrap();
    assert_eq!(
        session
            .windows()
            .get(peer)
            .unwrap()
            .cell(buffer)
            .unwrap()
            .row,
        9
    );
    assert_eq!(
        session.windows().get(peer).unwrap().visible_lines(buffer),
        12
    );
    session.execute(Command::Window(WindowCommand::Other));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), OUTLINE);
    assert_eq!(
        session.windows().get(origin).unwrap().point.position,
        before
    );
    assert_eq!(row(&session), 8);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), edited);
    assert_eq!(
        session.windows().get(origin).unwrap().point.position,
        inserted
    );
    assert_eq!(row(&session), 9);
}

// Pending edits have no authority after cancellation, source change or view
// change. The single worker is reaped, and a later fresh request can succeed.
#[test]
fn retired_insertions() {
    for event in ["cancel", "edit", "view"] {
        let (_fixture, mut session) = editor("syntax.enabled = false", "notes.org", OUTLINE);
        session.execute(Command::Split(Split::Right));
        session.execute(Command::Org(OrgCommand::MetaReturn));
        match event {
            "cancel" => {
                session.execute(Command::KeyboardQuit);
            }
            "edit" => {
                session.insert('!');
            }
            _ => {
                session.execute(Command::Window(WindowCommand::Other));
            }
        }
        settle(&mut session);
        assert_eq!(
            text(&session),
            if event == "edit" {
                format!("!{OUTLINE}")
            } else {
                OUTLINE.into()
            },
            "{event}"
        );
        assert_eq!(session.remaining_input(Instant::now()), None, "{event}");
    }
    let (_fixture, mut session) = editor("syntax.enabled = false", "notes.org", OUTLINE);
    action(&mut session, OrgCommand::MetaReturn);
    assert!(text(&session).starts_with("* \n* Root\n"));
    assert_eq!(session.remaining_input(Instant::now()), None);
    assert_eq!(visible(&session).len(), 12);
}
