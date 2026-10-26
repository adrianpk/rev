use rev::command::{Command, History, RectangleCommand};
use rev::coordinates::{Coordinates, Scalar};
use rev::session::{Action, Session};

use super::{cell, selected, text};

fn kill(session: &mut Session) {
    assert_eq!(
        session.execute(Command::Rectangle(RectangleCommand::Kill)),
        Action::Redraw
    );
    assert_eq!(session.echo(), "Rectangle killed");
}

// All changed rows use one base space and one history group. Partial tabs leave
// only their unselected cells; whole Unicode clusters and row separators survive undo.
#[test]
fn atomic_rows() {
    let source = "012345\r\na\t界z\r\nQ👩\u{200d}💻e\u{301}X\r\nx\r\n\r\n012345";
    let mut session = selected(source, cell(0, 2), cell(5, 5));
    let original = session.view().point.position;
    let mark = session.buffer().unwrap().mark();
    let revision = session.buffer().unwrap().stamp().revision;
    kill(&mut session);
    let killed = "015\r\na z\r\nQ\r\nx\r\n\r\n015";
    assert_eq!(text(&session), killed);
    assert_eq!(
        session.rectangle().unwrap().rows(),
        ["234", "  界", "👩\u{200d}💻e\u{301}X", "", "", "234"]
    );
    assert_eq!(session.rectangle().unwrap().width(), 3);
    assert_eq!(session.view().point.position, Scalar(2));
    assert!(!session.buffer().unwrap().rectangle_active());
    assert_eq!(session.buffer().unwrap().stamp().revision, revision + 1);
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
        4
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(session.view().point.position, original);
    assert_eq!(session.buffer().unwrap().mark(), mark);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), killed);
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(session.kill_ring().len(), 0);
}

// A partial leading tab on the top row leaves actual prefix spaces. The explicit
// initiating point follows them rather than inheriting a byte/scalar column.
#[test]
fn tab_cursor() {
    let source = "\txyz\n01234567";
    let mut session = selected(source, cell(0, 6), cell(1, 2));
    let before = session.view().point.position;
    kill(&mut session);
    assert_eq!(text(&session), "  z\n0167");
    assert_eq!(session.rectangle().unwrap().rows(), ["  xy", "2345"]);
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(
        Coordinates::new(session.buffer().unwrap())
            .unwrap()
            .to_cell(Scalar(2))
            .unwrap(),
        cell(0, 2)
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(session.view().point.position, before);
}

// The fixed-name handler and C-x r k reach the same real transaction; the key
// suffix is consumed and cannot insert a literal k or change the linear ring.
#[test]
fn command_routes() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    for named in [false, true] {
        let mut session = selected("abcdef\nabcdef", cell(0, 1), cell(1, 4));
        if named {
            session.execute(Command::ExecuteExtended);
            session.handle(Event::Paste("kill-rectangle".into()));
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
                KeyCode::Char('k'),
                KeyModifiers::NONE,
            )));
        }
        assert_eq!(session.echo(), "Rectangle killed");
        assert_eq!(text(&session), "aef\naef");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert_eq!(session.kill_ring().len(), 0);
    }
    assert_eq!(
        Command::named("kill-rectangle"),
        Some(Command::Rectangle(RectangleCommand::Kill))
    );
}
