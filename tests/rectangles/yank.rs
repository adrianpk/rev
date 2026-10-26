use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::{Buffers, edit::Point};
use rev::command::{BufferCommand, Command, History, RectangleCommand};
use rev::coordinates::{Cell, Coordinates};
use rev::display::Size;
use rev::session::{Action, Session};
use rev::view::View;

use super::{cell, text};

const SOURCE: &str = "0AB9\nx界z\n\n0e\u{301}X9";

fn retained(source: &str, endpoints: (Cell, Cell), target: &str, read_only: bool) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let source_id = buffers.create(source).unwrap();
    let target_id = buffers.create(target).unwrap();
    let target_buffer = buffers.get_mut(target_id).unwrap();
    target_buffer.set_read_only(read_only);
    let name = target_buffer.name().to_owned();
    let buffer = buffers.get_mut(source_id).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    let a = coordinates.from_cell(endpoints.0).unwrap();
    let b = coordinates.from_cell(endpoints.1).unwrap();
    buffer
        .toggle_rectangle(Point {
            stamp: buffer.stamp(),
            position: a,
        })
        .unwrap();
    let mut view = View::new(buffer);
    view.point.position = b;
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 48,
            height: 8,
        },
    )
    .unwrap();
    super::copy(&mut session);
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(Event::Paste(name));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(text(&session), target);
    session
}

fn yank(session: &mut Session) {
    assert_eq!(
        session.execute(Command::Rectangle(RectangleCommand::Yank)),
        Action::Redraw
    );
    assert_eq!(session.echo(), "Rectangle yanked");
}

// One transaction inserts unequal row fragments, adds missing lines and only
// the necessary prefixes. Empty copied rows advance without synthetic padding.
#[test]
fn append_rows() {
    let mut session = retained(SOURCE, (cell(0, 1), cell(3, 3)), "....", false);
    super::at(&mut session, cell(0, 3));
    let before = session.view().point.position;
    yank(&mut session);
    let expected = "...AB.\n   界\n\n   e\u{301}X";
    assert_eq!(text(&session), expected);
    assert_eq!(
        session.rectangle().unwrap().rows(),
        ["AB", "界", "", "e\u{301}X"]
    );
    assert_eq!(session.rectangle().unwrap().width(), 2);
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        cell(3, 5)
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .history()
            .back()
            .unwrap()
            .changes()
            .len(),
        2
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "....");
    assert_eq!(session.view().point.position, before);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
    assert_eq!(session.kill_ring().len(), 0);
}

// A target inside a wide/tab glyph snaps to its source start. Existing short
// rows pad only for nonempty fragments, while empty fragments leave rows intact.
#[test]
fn target_edges() {
    for (middle, inserted) in [("界Z", "界界Z"), ("\tZ", "界\tZ")] {
        let target = format!("abcde\n{middle}\n\tx\nq");
        let mut session = retained(SOURCE, (cell(0, 1), cell(3, 3)), &target, false);
        super::at(&mut session, cell(0, 1));
        yank(&mut session);
        assert_eq!(
            text(&session),
            format!("aABbcde\n{inserted}\n\tx\nqe\u{301}X")
        );
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap(),
            cell(3, 3)
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), target);
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// Bound and named entry share a real multi-row insertion. The separate retained
// block survives replay, and command suffixes cannot become document characters.
#[test]
fn command_routes() {
    for named in [false, true] {
        let mut session = retained("0AB9\n0CD9", (cell(0, 1), cell(1, 3)), "x\ny", false);
        if named {
            session.execute(Command::ExecuteExtended);
            session.handle(Event::Paste("yank-rectangle".into()));
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )));
        } else {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('x'),
                KeyModifiers::CONTROL,
            )));
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('r'),
                KeyModifiers::NONE,
            )));
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('y'),
                KeyModifiers::NONE,
            )));
        }
        assert_eq!(session.echo(), "Rectangle yanked");
        assert_eq!(text(&session), "ABx\nCDy");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "x\ny");
        assert_eq!(session.rectangle().unwrap().rows(), ["AB", "CD"]);
    }
    assert_eq!(
        Command::named("yank-rectangle"),
        Some(Command::Rectangle(RectangleCommand::Yank))
    );
}
