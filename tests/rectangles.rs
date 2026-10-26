#![forbid(unsafe_code)]

use rev::buffer::Buffers;
use rev::buffer::edit::Point;
use rev::command::{Command, Motion, RectangleCommand, RegionCommand};
use rev::coordinates::{Cell, Coordinates, Scalar};
use rev::display::Size;
use rev::session::{Action, Session};
use rev::view::View;

fn selected(text: &str, a: Cell, b: Cell) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(text).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    let start = coordinates.from_cell(a).unwrap();
    let end = coordinates.from_cell(b).unwrap();
    buffer
        .toggle_rectangle(Point {
            stamp: buffer.stamp(),
            position: start,
        })
        .unwrap();
    let mut view = View::new(buffer);
    view.point.position = end;
    Session::new(
        buffers,
        view,
        Size {
            width: 48,
            height: 8,
        },
    )
    .unwrap()
}

fn cell(row: usize, column: usize) -> Cell {
    Cell { row, column }
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn copy(session: &mut Session) {
    assert_eq!(
        session.execute(Command::Rectangle(RectangleCommand::Copy)),
        Action::Redraw
    );
    assert_eq!(session.echo(), "Rectangle copied");
}

// Rectangular extraction selects existing cells on every inclusive source row:
// partial tabs become spaces, while touched wide and combining clusters stay whole.
#[test]
fn cell_fragments() {
    let source = "012345\na\t界z\nQ👩\u{200d}💻e\u{301}X\nx\n\n012345";
    let mut session = selected(source, cell(0, 2), cell(5, 5));
    let stamp = session.buffer().unwrap().stamp();
    let point = session.view().point;
    copy(&mut session);
    let block = session.rectangle().unwrap();
    assert_eq!(
        block.rows(),
        ["234", "  界", "👩\u{200d}💻e\u{301}X", "", "", "234"]
    );
    assert_eq!(block.width(), 3);
    assert_eq!(text(&session), source);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.view().point, point);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(session.kill_ring().len(), 0);
}

// Toggle starts at point or reuses an already active mark; it changes only
// selection mode. Normal mark and C-g restore the linear/inactive interactions.
#[test]
fn mark_lifetime() {
    let mut session = selected("abcdef\nsecond", cell(0, 1), cell(1, 4));
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    assert!(!session.buffer().unwrap().mark_active());
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(11)));
    session.execute(Command::Motion(Motion::LineStart));
    session.execute(Command::Region(RegionCommand::Set));
    assert!(!session.buffer().unwrap().rectangle_active());
    session.execute(Command::Motion(Motion::LineEnd));
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(7)));
    assert!(session.buffer().unwrap().rectangle_active());
    session.execute(Command::KeyboardQuit);
    assert!(!session.buffer().unwrap().rectangle_active());
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(7)));
    assert!(session.buffer().unwrap().history().is_empty());
}

// Bound and named commands resolve to the same handlers; a nested Meta copy
// cannot escape to linear M-w, and a repeated prefix suffix is never inserted.
#[test]
fn command_routes() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::input::{Input, Outcome};
    let event = |code, modifiers| Event::Key(KeyEvent::new(code, modifiers));
    for meta in [KeyModifiers::ALT, KeyModifiers::NONE] {
        let mut input = Input::default();
        input.handle(event(KeyCode::Char('x'), KeyModifiers::CONTROL));
        assert_eq!(
            input.handle(event(KeyCode::Char(' '), KeyModifiers::NONE)),
            Outcome::Command(Command::Rectangle(RectangleCommand::Mark))
        );
        input.handle(event(KeyCode::Char('x'), KeyModifiers::CONTROL));
        input.handle(event(KeyCode::Char('r'), KeyModifiers::NONE));
        if meta.is_empty() {
            input.handle(event(KeyCode::Esc, KeyModifiers::NONE));
        }
        assert_eq!(
            input.handle(event(KeyCode::Char('w'), meta)),
            Outcome::Command(Command::Rectangle(RectangleCommand::Copy))
        );
    }
    assert_eq!(
        Command::named("rectangle-mark-mode"),
        Some(Command::Rectangle(RectangleCommand::Mark))
    );
    assert_eq!(
        Command::named("copy-rectangle-as-kill"),
        Some(Command::Rectangle(RectangleCommand::Copy))
    );
}
