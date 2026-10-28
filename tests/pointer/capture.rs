use crossterm::event::Event;
use rev::command::{Command, GutterCommand, Motion, Split, TreeCommand, WindowCommand};
use rev::coordinates::Scalar;
use rev::display::Position;
use rev::panel::Composition;
use rev::session::{Action, Session};

use super::scroll::{editor, size, wheel};
use super::selection::{drag, range, release};

// Capture stays with its originating leaf through decorations, gaps, other
// panes and wheel reports. Returning to source text resumes the same anchor.
#[test]
fn source_ownership() {
    for right in [false, true] {
        let mut session = editor("alpha\nbeta\n");
        session.execute(Command::Split(Split::Right));
        session.execute(Command::Gutter(GutterCommand::Relative));
        if right {
            session.execute(Command::Gutter(GutterCommand::ToggleSide));
        }
        let active = session.windows().selected_id();
        let peer = session.windows().order()[1];
        let other = session.windows().get(peer).unwrap().clone();
        let layout = Composition::workspace(session.windows(), session.tree(), size(), 0).unwrap();
        let text = session
            .view()
            .gutter_geometry(session.buffer().unwrap(), layout.panels()[0].1.content())
            .unwrap()
            .text();
        let origin = text.column();
        session.handle(super::focus::press(Position {
            column: origin + 1,
            row: 0,
        }));
        session.handle(drag(origin + 3, 1));
        let before = session.view().clone();
        let gutter = if right { 38 } else { 0 };
        for (column, row) in [
            (gutter, 0),
            (0, 10),
            (39, 0),
            (46, 1),
            (0, 11),
            (origin, 9),
            (u16::MAX, u16::MAX),
        ] {
            assert_eq!(session.handle(drag(column, row)), Action::Wait);
            assert_eq!(range(&session), Some(1..9));
            assert_eq!(session.view(), &before);
            assert_eq!(session.windows().get(peer).unwrap(), &other);
            assert_eq!(session.windows().selected_id(), active);
        }
        assert_eq!(session.handle(wheel(false, 46, 1)), Action::Wait);
        session.handle(drag(origin + 2, 1));
        assert_eq!(range(&session), Some(1..8));
        session.handle(release(46, 1));
        assert!(!session.cancel_pointer_capture());
        assert_eq!(range(&session), Some(1..8));
    }
}

fn cancel(session: &mut Session, kind: &str) {
    match kind {
        "quit" => {
            session.execute(Command::KeyboardQuit);
        }
        "resize" => {
            session.handle(Event::Resize(80, 12));
        }
        "lost focus" => {
            session.handle(Event::FocusLost);
        }
        "mode owner" => {
            assert!(session.cancel_pointer_capture());
        }
        "command" => {
            session.execute(Command::Motion(Motion::BufferStart));
        }
        "edit" => {
            session.replace(0..0, "Z").unwrap();
        }
        "close" => {
            session.execute(Command::Window(WindowCommand::Delete));
        }
        "switch" => {
            session.select_view(session.windows().order()[1]).unwrap();
        }
        _ => {
            session.execute(Command::ExecuteExtended);
        }
    }
}

// Cancellation cannot be undone by delayed motion/release. Real commands,
// edits, view loss and the native mode-owner hook retain their own semantics.
#[test]
fn ownership_loss() {
    for kind in [
        "quit",
        "resize",
        "lost focus",
        "mode owner",
        "command",
        "edit",
        "close",
        "switch",
        "prompt",
    ] {
        let mut session = editor("alpha\nbeta\n");
        session.execute(Command::Split(Split::Right));
        session.handle(super::focus::press(Position { column: 7, row: 0 }));
        session.handle(drag(9, 1));
        cancel(&mut session, kind);
        assert!(!session.cancel_pointer_capture(), "{kind}");
        let points = session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect::<Vec<_>>();
        let selected = session.windows().selected_id();
        let mark = session.buffer().unwrap().mark();
        let active = session.buffer().unwrap().mark_active();
        assert_eq!(session.handle(drag(10, 0)), Action::Wait, "{kind}");
        session.handle(release(6, 0));
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(id, view)| (id, view.clone()))
                .collect::<Vec<_>>(),
            points
        );
        assert_eq!(session.windows().selected_id(), selected);
        assert_eq!(
            (
                session.buffer().unwrap().mark(),
                session.buffer().unwrap().mark_active()
            ),
            (mark, active)
        );
        if kind == "quit" {
            assert!(!active);
        }
        if kind == "mode owner" {
            assert!(active);
        }
    }
}

// Non-text presses never arm a text drag; tree focus also remains its owner.
#[test]
fn non_text_origin() {
    for position in [
        Position { column: 0, row: 0 },
        Position { column: 0, row: 10 },
        Position { column: 6, row: 9 },
    ] {
        let mut session = editor("alpha\nbeta\n");
        session.handle(super::focus::press(position));
        assert_eq!(session.handle(drag(9, 1)), Action::Wait);
        assert_eq!(session.view().point.position, Scalar(0));
        assert_eq!(range(&session), None);
    }
    let mut session = editor("alpha\nbeta\n");
    session.execute(Command::Tree(TreeCommand::Toggle));
    session.handle(super::focus::press(Position { column: 1, row: 0 }));
    session.handle(drag(40, 1));
    assert!(session.tree().focused());
    assert_eq!(session.view().point.position, Scalar(0));
}

// A successful source rebind retires capture even when the old view stays live.
#[test]
fn source_rebind() {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.write("other.txt", "other source\r\n");
    let mut session = editor("alpha\nbeta\n");
    session.handle(super::focus::press(Position { column: 7, row: 0 }));
    session.handle(drag(9, 1));
    session.load(&path).unwrap();
    let before = session.view().clone();
    assert_eq!(session.handle(drag(10, 0)), Action::Wait);
    assert_eq!(session.view(), &before);
    assert_eq!(range(&session), None);
    assert!(session.buffer().unwrap().history().is_empty());
}
