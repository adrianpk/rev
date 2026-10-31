use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{
    BufferCommand, Command, GutterCommand, History, RegionCommand, TreeCommand, WindowCommand,
};
use rev::coordinates::Scalar;
use rev::display::{Position, Size};
use rev::panel::Composition;
use rev::session::{Action, Session};
use rev::view::View;
use rev::window::{Split, ViewId};

use super::focus::press;

fn size() -> Size {
    Size {
        width: 80,
        height: 12,
    }
}

fn editor(read_only: bool) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("alpha\nbeta\n").unwrap();
    buffers.get_mut(id).unwrap().set_readonly(read_only);
    let view = View::new(buffers.get(id).unwrap());
    Session::new(buffers, view, size()).unwrap()
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn text_cell(session: &Session, id: ViewId, size: Size) -> Position {
    let layout = Composition::workspace(session.windows(), session.tree(), size, 0).unwrap();
    let panel = layout
        .panels()
        .iter()
        .find(|(view, _)| *view == id)
        .unwrap()
        .1;
    let view = session.windows().get(id).unwrap();
    // The selected view shares this buffer in these geometry fixtures.
    let area = view
        .gutter_geometry(session.buffer().unwrap(), panel.content())
        .unwrap()
        .text();
    Position {
        column: area.column(),
        row: area.row(),
    }
}

fn source(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Tree ownership changes through the normal focus transition. A click does
// not activate an entry, enumerate the directory or change a document point.
#[test]
fn tree_focus() {
    let mut session = editor(false);
    let id = session.windows().selected_id();
    session.execute(Command::Tree(TreeCommand::Toggle));
    let entries = session.tree().entries().len();
    let root = session.tree().root().unwrap().to_owned();
    let target = text_cell(&session, id, size());
    assert_eq!(
        session.handle(press(Position {
            column: target.column + 1,
            ..target
        })),
        Action::Redraw
    );
    assert!(!session.tree().focused());
    assert_eq!(session.view().point.position, Scalar(1));
    assert_eq!(
        session.handle(press(Position { column: 1, row: 0 })),
        Action::Redraw
    );
    assert!(session.tree().focused());
    assert_eq!(session.view().point.position, Scalar(1));
    assert_eq!(session.tree().entries().len(), entries);
    assert_eq!(session.tree().root(), Some(root.as_path()));
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("tree".into()));
    assert_eq!(session.handle(press(target)), Action::Wait);
    assert!(session.tree().focused());
    session.execute(Command::KeyboardQuit);
    assert!(session.tree().focused());
    session.handle(press(target));
    assert!(!session.tree().focused());
    session.handle(key(KeyCode::Char('Z')));
    assert_eq!(source(&session), "Zalpha\nbeta\n");
}

// A cell from an older projection belongs to the new layout after resize.
// Closing a leaf and suspending tiny/zero geometry cannot retain a hit target.
#[test]
fn layout_changes() {
    let mut session = editor(false);
    let original = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    let old = text_cell(&session, peer, size());
    session.select_view(peer).unwrap();
    session.handle(Event::Resize(120, 12));
    session.handle(press(old));
    assert_eq!(session.windows().selected_id(), original);
    assert_eq!(session.view().point.position, Scalar(0));
    session.select_view(peer).unwrap();
    session.execute(Command::Window(WindowCommand::Delete));
    assert!(session.windows().get(peer).is_err());
    session.handle(press(Position { column: 7, row: 1 }));
    assert_eq!(session.windows().selected_id(), original);
    assert_eq!(session.view().point.position, Scalar(7));
    let before = session.view().clone();
    for (width, height) in [(0, 0), (1, 1), (2, 2)] {
        session.handle(Event::Resize(width, height));
        assert_eq!(
            session.handle(press(Position { column: 0, row: 0 })),
            Action::Wait
        );
        assert_eq!(session.view(), &before);
    }
    session.handle(Event::Resize(80, 12));
    assert_eq!(
        session.frame().unwrap().cursor(),
        Some(Position { column: 7, row: 1 })
    );
}

// A nested right-sided relative gutter and a scrolled viewport use the live
// text origin. Other shared points and canonical source remain independent.
#[test]
fn nested_origins() {
    let mut session = editor(false);
    let original = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let right = session.windows().order()[1];
    session.select_view(right).unwrap();
    session.execute(Command::Split(Split::Below));
    let bottom = session.windows().order()[2];
    session.select_view(bottom).unwrap();
    session.execute(Command::Gutter(GutterCommand::Relative));
    session.execute(Command::Gutter(GutterCommand::ToggleSide));
    let target = text_cell(&session, bottom, size());
    session.select_view(original).unwrap();
    session.handle(press(Position {
        column: target.column + 2,
        row: target.row + 1,
    }));
    assert_eq!(session.windows().selected_id(), bottom);
    assert_eq!(session.view().point.position, Scalar(8));
    assert_eq!(
        session.frame().unwrap().cursor(),
        Some(Position {
            column: target.column + 2,
            row: target.row + 1
        })
    );
    assert_eq!(
        session.windows().get(right).unwrap().point.position,
        Scalar(0)
    );
    assert_eq!(
        session.windows().get(original).unwrap().point.position,
        Scalar(0)
    );
    assert!(session.buffer().unwrap().history().is_empty());
}

// Different files remain bound to their own live leaves. Source revisions,
// saved bytes and history do not change when the pointer changes focus.
#[test]
fn different_buffers() {
    let fixture = super::config_fixture::Fixture::new();
    let first = fixture.write("one.txt", "one\r\n");
    let second = fixture.write("two.txt", "界e\u{301}\r\n");
    let mut buffers = Buffers::new().unwrap();
    let a = buffers.visit(&first).unwrap();
    buffers.visit(&second).unwrap();
    let view = View::new(buffers.get(a).unwrap());
    let mut session = Session::new(buffers, view, size()).unwrap();
    let original = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    session.select_view(peer).unwrap();
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(Event::Paste("two.txt".into()));
    session.handle(key(KeyCode::Enter));
    let b = session.buffer().unwrap().stamp();
    session.handle(press(Position { column: 7, row: 0 }));
    assert_eq!(session.windows().selected_id(), original);
    assert_eq!(session.view().point.position, Scalar(1));
    // The right pane starts at cell 40; six gutter cells precede the wide glyph.
    session.handle(press(Position { column: 48, row: 0 }));
    assert_eq!(session.windows().selected_id(), peer);
    assert_eq!(session.buffer().unwrap().stamp(), b);
    assert_eq!(session.view().point.position, Scalar(1));
    assert_eq!(source(&session), "界e\u{301}\r\n");
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(std::fs::read(first).unwrap(), b"one\r\n");
    assert_eq!(std::fs::read(second).unwrap(), "界e\u{301}\r\n".as_bytes());
}

// Clicking is ordinary movement for active regions, including read-only text.
// It keeps the mark, grows canonical selection, and never records an edit.
#[test]
fn region_movement() {
    for read_only in [false, true] {
        let mut session = editor(read_only);
        session.execute(Command::Region(RegionCommand::Set));
        let before = session.buffer().unwrap().stamp();
        session.handle(press(Position { column: 8, row: 1 }));
        let buffer = session.buffer().unwrap();
        assert_eq!(buffer.mark(), Some(Scalar(0)));
        assert!(buffer.mark_active());
        assert_eq!(
            buffer.region(session.view().point).unwrap().unwrap().range,
            0..8
        );
        assert_eq!(buffer.stamp(), before);
        assert!(buffer.history().is_empty());
        let frame = session.frame().unwrap();
        assert_eq!(frame.rows()[0].selection(), Some(6..80));
        assert_eq!(frame.rows()[1].selection(), Some(6..8));
        session.execute(Command::KeyboardQuit);
        session.handle(press(Position { column: 7, row: 0 }));
        assert!(!session.buffer().unwrap().mark_active());
        assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(0)));
        assert_eq!(source(&session), "alpha\nbeta\n");
    }
}

// A press is a typing boundary, without introducing a history item of its own.
// Undo/redo recover the exact ordered edits around the moved point.
#[test]
fn typing_boundary() {
    let mut session = editor(false);
    session.handle(key(KeyCode::Char('X')));
    session.handle(press(Position { column: 8, row: 1 }));
    session.handle(key(KeyCode::Char('Y')));
    assert_eq!(source(&session), "Xalpha\nbeYta\n");
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    session.execute(Command::History(History::Undo));
    assert_eq!(source(&session), "Xalpha\nbeta\n");
    session.execute(Command::History(History::Undo));
    assert_eq!(source(&session), "alpha\nbeta\n");
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    session.execute(Command::History(History::Redo));
    assert_eq!(source(&session), "Xalpha\nbeYta\n");
}

// Keyboard navigation creates nonzero horizontal and vertical offsets. The
// next press uses those current offsets and leaves the viewport stable.
#[test]
fn scrolled_text() {
    let text = (0..50)
        .map(|row| format!("{row:02}:{}\n", "x".repeat(100)))
        .collect::<String>();
    for wrap in [true, false] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(&text).unwrap();
        if !wrap {
            buffers.get_mut(id).unwrap().toggle_soft_wrap();
        }
        let view = View::new(buffers.get(id).unwrap());
        let mut session = Session::new(buffers, view, size()).unwrap();
        session.execute(Command::GotoLine);
        session.handle(Event::Paste("25".into()));
        session.handle(key(KeyCode::Enter));
        session.execute(Command::Motion(rev::command::Motion::LineEnd));
        let (top, left) = (session.view().top, session.view().left);
        assert!(top > 0);
        assert_eq!(left > 0, !wrap);
        assert_eq!(
            session.handle(press(Position { column: 8, row: 1 })),
            Action::Redraw
        );
        assert_eq!(
            session.view().point.position,
            Scalar(if wrap {
                (top + 1) * 104 + 2
            } else {
                (top + 1) * 104 + left + 2
            })
        );
        assert_eq!((session.view().top, session.view().left), (top, left));
        assert_eq!(
            session.frame().unwrap().cursor(),
            Some(Position { column: 8, row: 1 })
        );
        assert_eq!(source(&session), text);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Prompt/decision/inventory owners ignore presses/wheels over text and auxiliary
// rows. Draft, cursor, focus, canonical point and cancellation remain theirs.
#[test]
fn prompt_isolation() {
    for command in [
        Command::ExecuteExtended,
        Command::FindFile,
        Command::GotoLine,
        Command::SearchForward,
        Command::WriteFile,
        Command::Buffer(BufferCommand::List),
        Command::Quit,
    ] {
        let mut session = editor(false);
        if command == Command::Quit {
            session.insert('X');
        }
        session.execute(command);
        if !matches!(command, Command::Quit | Command::Buffer(_)) {
            session.handle(Event::Paste("2".into()));
        }
        let before = session.view().point;
        let text = source(&session);
        let frame = session.frame().unwrap();
        for event in [
            press(Position { column: 8, row: 1 }),
            press(Position { column: 1, row: 11 }),
            super::scroll::wheel(true, 8, 1),
            super::scroll::wheel(false, 8, 1),
        ] {
            assert_eq!(session.handle(event), Action::Wait, "{command:?}");
            let after = session.frame().unwrap();
            assert_eq!(
                after
                    .rows()
                    .iter()
                    .map(|row| row.text())
                    .collect::<Vec<_>>(),
                frame
                    .rows()
                    .iter()
                    .map(|row| row.text())
                    .collect::<Vec<_>>(),
                "{command:?}"
            );
            assert_eq!(after.cursor(), frame.cursor(), "{command:?}");
            assert_eq!(session.view().point, before);
            assert_eq!(source(&session), text);
        }
        session.execute(Command::KeyboardQuit);
        session.handle(press(Position { column: 7, row: 0 }));
        assert_eq!(session.view().point.position, Scalar(1));
    }
}
